use std::{
    convert::Infallible,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

use futures::ready;
use http_body::combinators::BoxBody;
use hyper::body::Bytes;
use pin_project_lite::pin_project;
use tower::{Layer, Service};

use super::{OperationError, OperationShape};

/// A protocol and operation aware conversion from [`http`] types to Smithy types.
pub trait FromRequest<Protocol, Operation, B>: Sized {
    /// Conversion failure.
    type Error: IntoResponse<Protocol, Operation>;
    type Future: Future<Output = Result<Self, Self::Error>>;

    fn from_request(request: http::Request<B>) -> Self::Future;
}

/// A protocol and operation aware conversion from Smithy types to [`http`] types.
pub trait IntoResponse<Protocol = (), Operation = ()>: Sized {
    fn into_response(self) -> http::Response<BoxBody<Bytes, hyper::Error>>;
}

impl<P, Op> IntoResponse<P, Op> for Infallible {
    fn into_response(self) -> http::Response<BoxBody<Bytes, hyper::Error>> {
        match self {}
    }
}

/// A [`Layer`] responsible for taking an operation [`Service`], accepting and returning Smithy
/// types and converting it into a [`Service`] taking and returning [`http`] types.
///
/// See [`Upgrade`].
#[derive(Debug, Clone)]
pub struct UpgradeLayer<Protocol, Operation, B> {
    _protocol: PhantomData<Protocol>,
    _operation: PhantomData<Operation>,
    _body: PhantomData<B>,
}

impl<Protocol, Operation, B> Default for UpgradeLayer<Protocol, Operation, B> {
    fn default() -> Self {
        Self {
            _protocol: PhantomData,
            _operation: PhantomData,
            _body: PhantomData,
        }
    }
}

impl<Protocol, Operation, B> UpgradeLayer<Protocol, Operation, B> {
    /// Creates a new [`UpgradeLayer`].
    pub fn new() -> Self {
        Self::default()
    }
}

impl<S, P, Op, B> Layer<S> for UpgradeLayer<P, Op, B> {
    type Service = Upgrade<P, Op, B, S>;

    fn layer(&self, inner: S) -> Self::Service {
        Upgrade {
            _protocol: PhantomData,
            _operation: PhantomData,
            _body: PhantomData,
            inner,
        }
    }
}

/// A alias allowing for quick access to [`UpgradeLayer`]s target [`Service`].
pub type UpgradedService<P, Op, B, S> = <UpgradeLayer<P, Op, B> as Layer<S>>::Service;

/// A [`Service`] responsible for wrapping an operation [`Service`] accepting and returning Smithy
/// types, and converting it into a [`Service`] accepting and returning [`http`] types.
pub struct Upgrade<Protocol, Operation, B, S> {
    _protocol: PhantomData<Protocol>,
    _operation: PhantomData<Operation>,
    _body: PhantomData<B>,
    inner: S,
}

impl<P, Op, B, S> Clone for Upgrade<P, Op, B, S>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            _protocol: PhantomData,
            _operation: PhantomData,
            _body: PhantomData,
            inner: self.inner.clone(),
        }
    }
}

pin_project! {
    /// The [`Service::Future`] of [`Upgrade`].
    pub struct UpgradeFuture<Protocol, Operation, B, S>
    where
        Operation: OperationShape,
        Operation::Input: FromRequest<Protocol, Operation, B>,
        S: Service<Operation::Input>,
    {
        service: S,
        #[pin]
        inner: Inner<<Operation::Input as FromRequest<Protocol, Operation, B>>::Future, S::Future>
    }
}

pin_project! {
    #[project = InnerProj]
    #[project_replace = InnerProjReplace]
    enum Inner<FromFut, HandlerFut> {
        FromRequest {
            #[pin]
            inner: FromFut
        },
        Inner {
            #[pin]
            call: HandlerFut
        }
    }
}

impl<P, Op, B, S, PollError, OpError> Future for UpgradeFuture<P, Op, B, S>
where
    // Op is used to specify the operation shape
    Op: OperationShape,
    // Smithy input must be convert from a HTTP request
    Op::Input: FromRequest<P, Op, B>,
    // Smithy output must be convert into a HTTP response
    Op::Output: IntoResponse<P, Op>,
    // Smithy error must convert into a HTTP response
    OpError: IntoResponse<P, Op>,

    // The signature of the inner service is correct
    S: Service<Op::Input, Response = Op::Output, Error = OperationError<PollError, OpError>>
        + Clone,
{
    type Output = Result<http::Response<BoxBody<Bytes, hyper::Error>>, PollError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            let mut this = self.as_mut().project();
            let this2 = this.inner.as_mut().project();

            let call = match this2 {
                InnerProj::FromRequest { inner } => {
                    let result = ready!(inner.poll(cx));
                    match result {
                        Ok(ok) => this.service.call(ok),
                        Err(err) => return Poll::Ready(Ok(err.into_response())),
                    }
                }
                InnerProj::Inner { call } => {
                    let result = ready!(call.poll(cx));
                    let output = match result {
                        Ok(ok) => ok.into_response(),
                        Err(OperationError::Smithy(err)) => err.into_response(),
                        Err(OperationError::Poll(_)) => {
                            unreachable!("poll error should not be raised")
                        }
                    };
                    return Poll::Ready(Ok(output));
                }
            };

            this.inner.as_mut().project_replace(Inner::Inner { call });
        }
    }
}

impl<P, Op, B, S, PollError, OpError> Service<http::Request<B>> for Upgrade<P, Op, B, S>
where
    Op: OperationShape,
    Op::Input: FromRequest<P, Op, B>,
    Op::Output: IntoResponse<P, Op>,
    OpError: IntoResponse<P, Op>,
    S: Service<Op::Input, Response = Op::Output, Error = OperationError<PollError, OpError>>
        + Clone,
{
    type Response = http::Response<BoxBody<Bytes, hyper::Error>>;
    type Error = PollError;
    type Future = UpgradeFuture<P, Op, B, S>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(|err| match err {
            OperationError::Poll(err) => err,
            OperationError::Smithy(_) => unreachable!("operation error should not be raised"),
        })
    }

    fn call(&mut self, req: http::Request<B>) -> Self::Future {
        UpgradeFuture {
            service: self.inner.clone(),
            inner: Inner::FromRequest {
                inner: <Op::Input as FromRequest<P, Op, B>>::from_request(req),
            },
        }
    }
}

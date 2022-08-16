use std::{
    convert::Infallible,
    future::Future,
    marker::PhantomData,
    task::{Context, Poll},
};

use futures::{
    future::{Map, MapErr},
    FutureExt, TryFutureExt,
};
use tower::Service;

/// The operation [`Service`] has two classes of failure modes - the failure models specified by
/// the Smithy model and failures to [`Service::poll_ready`].
pub enum OperationError<PollError, SmithyError> {
    /// A [`Service::poll_ready`] failure occured.
    Poll(PollError),
    /// An error modelled by the Smithy model occured.
    Smithy(SmithyError),
}

/// A utility trait used to provide an even interface for all handlers.
pub trait Handler<Input, Output, Error> {
    type Future: Future<Output = Result<Output, Error>>;

    fn call(&mut self, req: Input) -> Self::Future;
}

/// A utility trait used to provide an even interface over return types `Result<Ok, Error>`/`Ok`.
trait ToResult<Ok, Error> {
    fn into_result(self) -> Result<Ok, Error>;
}

// We can convert from `Result<Ok, Error>` to `Result<Ok, Error>`.
impl<Ok, Error> ToResult<Ok, Error> for Result<Ok, Error> {
    fn into_result(self) -> Result<Ok, Error> {
        self
    }
}

// We can convert from `Ok` to `Result<Ok, Error>`.
impl<Ok> ToResult<Ok, Infallible> for Ok {
    fn into_result(self) -> Result<Ok, Infallible> {
        Ok(self)
    }
}

// fn(Input) -> Output
impl<Input, Output, Error, F, Fut> Handler<Input, Output, Error> for F
where
    F: FnMut(Input) -> Fut,
    Fut: Future,
    Fut::Output: ToResult<Output, Error>,
{
    type Future = Map<Fut, fn(Fut::Output) -> Result<Output, Error>>;

    fn call(&mut self, req: Input) -> Self::Future {
        (self)(req).map(ToResult::into_result)
    }
}

/// Adjoins state to a `fn(Input, State) -> Output` to create a [`Handler`].
#[derive(Clone)]
pub struct StatefulHandler<F, T> {
    f: F,
    state: T,
}

// fn(Input, State) -> Output
impl<Input, Output, Error, F, Fut, T> Handler<Input, Output, Error> for StatefulHandler<F, T>
where
    T: Clone,
    F: Fn(Input, T) -> Fut,
    Fut: Future,
    Fut::Output: ToResult<Output, Error>,
{
    type Future = Map<Fut, fn(Fut::Output) -> Result<Output, Error>>;

    fn call(&mut self, req: Input) -> Self::Future {
        let Self { f, state } = self;
        f(req, state.clone()).map(ToResult::into_result)
    }
}

/// Provides the ability to [`AdjoinState::with_state`] on closures of the form
/// `(Input, State) -> Output` converting them to a [`StatefulHandler`] and therefore causing them
/// to implement [`Handler`].
pub trait AdjoinState {
    fn with_state<T>(self, state: T) -> StatefulHandler<Self, T>
    where
        Self: Sized,
    {
        StatefulHandler { f: self, state }
    }
}

impl<F> AdjoinState for F {}

/// An extension trait for [`Handler`].
pub trait HandlerExt<Input, Output, Error>: Handler<Input, Output, Error> {
    /// Convert the [`Handler`] into a [`Service`].
    fn into_service(self) -> IntoService<Output, Error, Self>
    where
        Self: Sized,
    {
        IntoService {
            handler: self,
            _error: PhantomData,
            _output: PhantomData,
        }
    }
}

impl<Input, Output, Error, H> HandlerExt<Input, Output, Error> for H where
    H: Handler<Input, Output, Error>
{
}

/// A [`Service`] provided for every [`Handler`].
pub struct IntoService<Output, Error, H> {
    handler: H,
    _output: PhantomData<Output>,
    _error: PhantomData<Error>,
}

impl<Output, Error, H> Clone for IntoService<Output, Error, H>
where
    H: Clone,
{
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
            _output: PhantomData,
            _error: PhantomData,
        }
    }
}

impl<Input, Output, Error, H> Service<Input> for IntoService<Output, Error, H>
where
    H: Handler<Input, Output, Error>,
{
    type Response = Output;
    type Error = OperationError<Infallible, Error>;
    type Future = MapErr<H::Future, fn(Error) -> Self::Error>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Input) -> Self::Future {
        self.handler.call(req).map_err(OperationError::Smithy)
    }
}

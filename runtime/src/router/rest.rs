use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use http_body::combinators::BoxBody;
use hyper::{body::Bytes, Error};
use pin_project_lite::pin_project;
use tower::{util::Oneshot, Layer, Service, ServiceExt};

use crate::{operation::IntoResponse, service::ServiceError};

#[derive(Clone)]
pub struct Router<S> {
    inner: HashMap<&'static str, S>,
}

impl<S> FromIterator<(&'static str, S)> for Router<S> {
    fn from_iter<T: IntoIterator<Item = (&'static str, S)>>(iter: T) -> Self {
        Self {
            inner: HashMap::from_iter(iter),
        }
    }
}

impl<S> Router<S> {
    /// Apply a [`Layer`] uniformly across all routes.
    pub fn layer<L>(self, layer: L) -> Router<L::Service>
    where
        L: Layer<S>,
    {
        Router {
            inner: self
                .inner
                .into_iter()
                .map(|(path, svc)| (path, layer.layer(svc)))
                .collect(),
        }
    }
}

pin_project! {
    pub struct RoutingFuture<S, Req> where S: Service<Req> {
        #[pin]
        inner: Inner<Oneshot<S, Req>, RoutingError>
    }
}

impl<S, Req> Future for RoutingFuture<S, Req>
where
    S: Service<Req>,
{
    type Output = Result<S::Response, ServiceError<RoutingError, S::Error>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let this2 = this.inner.project();
        match this2 {
            InnerProj::Future { value } => value.poll(cx).map_err(ServiceError::Poll),
            InnerProj::Ready { value } => {
                let error = value
                    .take()
                    .expect("RoutingFuture cannot be polled after completion");
                Poll::Ready(Err(ServiceError::Routing(error)))
            }
        }
    }
}

pin_project! {
    #[project = InnerProj]
    enum Inner<Fut, Error> {
        Future {
            #[pin]
            value: Fut
        },
        Ready { value: Option<Error> }
    }
}

pub enum RoutingError {
    Missing,
}

impl IntoResponse for RoutingError {
    fn into_response(self) -> http::Response<BoxBody<Bytes, Error>> {
        todo!()
    }
}

impl<B, S> Service<http::Request<B>> for Router<S>
where
    S: Service<http::Request<B>> + Clone,
{
    type Response = S::Response;
    type Error = ServiceError<RoutingError, S::Error>;
    type Future = RoutingFuture<S, http::Request<B>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<B>) -> Self::Future {
        let inner = match self.inner.get_mut(req.uri().path()) {
            Some(svc) => Inner::Future {
                value: svc.clone().oneshot(req),
            },
            None => Inner::Ready {
                value: Some(RoutingError::Missing),
            },
        };
        RoutingFuture { inner }
    }
}

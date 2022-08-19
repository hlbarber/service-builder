use std::{convert::Infallible, future::Ready};

use futures::{
    future::{MapErr, MapOk, TryJoin},
    Future, TryFutureExt,
};
use http_body::combinators::BoxBody;
use hyper::body::Bytes;

/// A protocol and operation aware conversion from [`http`] types to Smithy types.
pub trait FromRequest<Protocol, Operation, B>: Sized {
    /// Conversion failure.
    type Error: IntoResponse<Protocol, Operation>;
    type Future: Future<Output = Result<Self, Self::Error>>;

    fn from_request(request: &mut http::Request<B>) -> Self::Future;
}

impl<P, Op, B> FromRequest<P, Op, B> for () {
    type Error = Infallible;
    type Future = Ready<Result<(), Infallible>>;

    fn from_request(_request: &mut http::Request<B>) -> Self::Future {
        std::future::ready(Ok(()))
    }
}

impl<P, Op, B, Arg0> FromRequest<P, Op, B> for (Arg0,)
where
    Arg0: FromRequest<P, Op, B>,
{
    type Error = Arg0::Error;
    type Future = MapOk<Arg0::Future, fn(Arg0) -> Self>;

    fn from_request(request: &mut http::Request<B>) -> Self::Future {
        Arg0::from_request(request).map_ok(|arg0| (arg0,))
    }
}

/// An wrapper for state stored in the [`http::Extensions`] map.
pub struct Extension<T>(pub T);

impl<P, Op, B, T> FromRequest<P, Op, B> for Extension<T>
where
    T: Sync + Send + 'static,
{
    type Error = Infallible;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(request: &mut http::Request<B>) -> Self::Future {
        std::future::ready(Ok(Extension(
            request
                .extensions_mut()
                .remove()
                .expect("request does not contain extension"),
        )))
    }
}

/// Represents one of two errors.
/// 
/// Implements [`IntoResponse`] if both inner also implement it.
pub enum Either<A, B> {
    Left(A),
    Right(B),
}

impl<A, B, P, Op> IntoResponse<P, Op> for Either<A, B>
where
    A: IntoResponse<P, Op>,
    B: IntoResponse<P, Op>,
{
    fn into_response(self) -> http::Response<BoxBody<Bytes, hyper::Error>> {
        match self {
            Either::Left(left) => left.into_response(),
            Either::Right(right) => right.into_response(),
        }
    }
}

impl<P, Op, B, Arg0, Arg1> FromRequest<P, Op, B> for (Arg0, Arg1)
where
    Arg0: FromRequest<P, Op, B>,
    Arg1: FromRequest<P, Op, B>,
{
    type Error = Either<Arg0::Error, Arg1::Error>;

    type Future = TryJoin<
        MapErr<Arg0::Future, fn(Arg0::Error) -> Self::Error>,
        MapErr<Arg1::Future, fn(Arg1::Error) -> Self::Error>,
    >;

    fn from_request(_request: &mut http::Request<B>) -> Self::Future {
        todo!()
    }
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

use std::{convert::Infallible, future::Ready};

use http_body::combinators::BoxBody;
use hyper::body::Bytes;
use runtime::{
    operation::{FromRequest, IntoResponse, OperationShape},
    protocols::AWSRestJsonV1,
};

use crate::structures::{EmptyOperationInput, EmptyOperationOutput};

pub struct EmptyOperation;

impl OperationShape for EmptyOperation {
    const NAME: &'static str = "EmptyOperation";

    type Input = EmptyOperationInput;
    type Output = EmptyOperationOutput;
    type Error = Infallible;
}

pub struct FromRequestError;

impl IntoResponse<AWSRestJsonV1, EmptyOperation> for FromRequestError {
    fn into_response(self) -> http::Response<BoxBody<Bytes, hyper::Error>> {
        todo!()
    }
}

impl<B> FromRequest<AWSRestJsonV1, EmptyOperation, B> for EmptyOperationInput {
    type Error = FromRequestError;

    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(_request: &mut http::Request<B>) -> Self::Future {
        todo!()
    }
}

impl IntoResponse<AWSRestJsonV1, EmptyOperation> for EmptyOperationOutput {
    fn into_response(self) -> http::Response<BoxBody<Bytes, hyper::Error>> {
        todo!()
    }
}

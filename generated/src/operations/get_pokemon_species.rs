use std::future::Ready;

use http_body::combinators::BoxBody;
use hyper::body::Bytes;
use runtime::{
    operation::{FromRequest, IntoResponse, OperationShape},
    protocols::AWSRestJsonV1,
};

use crate::structures::{
    GetPokemonSpeciesInput, GetPokemonSpeciesOutput, ResourceNotFoundException,
};

pub struct GetPokemonSpecies;

impl OperationShape for GetPokemonSpecies {
    const NAME: &'static str = "GetPokemonSpecies";

    type Input = GetPokemonSpeciesInput;
    type Output = GetPokemonSpeciesOutput;
    type Error = ResourceNotFoundException;
}

pub struct FromRequestError;

impl IntoResponse<AWSRestJsonV1, GetPokemonSpecies> for FromRequestError {
    fn into_response(self) -> http::Response<BoxBody<Bytes, hyper::Error>> {
        todo!()
    }
}

impl<B> FromRequest<AWSRestJsonV1, GetPokemonSpecies, B> for GetPokemonSpeciesInput {
    type Error = FromRequestError;

    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(_request: &mut http::Request<B>) -> Self::Future {
        todo!()
    }
}

impl IntoResponse<AWSRestJsonV1, GetPokemonSpecies> for GetPokemonSpeciesOutput {
    fn into_response(self) -> http::Response<BoxBody<Bytes, hyper::Error>> {
        todo!()
    }
}

impl IntoResponse<AWSRestJsonV1, GetPokemonSpecies> for ResourceNotFoundException {
    fn into_response(self) -> http::Response<BoxBody<Bytes, hyper::Error>> {
        todo!()
    }
}

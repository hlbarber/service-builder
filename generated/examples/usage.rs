use std::{convert::Infallible, future::Ready, task::Poll};

use generated::{operations::*, services::*, structures::*};
use runtime::operation::{Extension, OperationError, OperationShapeExt};
use tower::{util::MapResponseLayer, Service};

// Fallible handler with extensions.
async fn get_pokemon_species_stateful(
    _input: GetPokemonSpeciesInput,
    _ext_a: Extension<usize>,
    _ext_b: Extension<String>,
) -> Result<GetPokemonSpeciesOutput, ResourceNotFoundException> {
    todo!()
}

// Fallible handler without extensions.
async fn get_pokemon_species(
    _input: GetPokemonSpeciesInput,
) -> Result<GetPokemonSpeciesOutput, ResourceNotFoundException> {
    todo!()
}

// Infallible handler without extensions.
async fn empty_operation(_input: EmptyOperationInput) -> EmptyOperationOutput {
    todo!()
}

// Bespoke implementation of `EmptyOperation`.
#[derive(Clone)]
struct EmptyOperationServiceA;

impl Service<EmptyOperationInput> for EmptyOperationServiceA {
    type Response = EmptyOperationOutput;
    type Error = OperationError<String, Infallible>;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        todo!()
    }

    fn call(&mut self, _req: EmptyOperationInput) -> Self::Future {
        todo!()
    }
}

// Bespoke implementation of `EmptyOperation` with an extension.
#[derive(Clone)]
struct EmptyOperationServiceB;

impl Service<(EmptyOperationInput, Extension<String>)> for EmptyOperationServiceB {
    type Response = EmptyOperationOutput;
    type Error = OperationError<String, Infallible>;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        todo!()
    }

    fn call(&mut self, _req: (EmptyOperationInput, Extension<String>)) -> Self::Future {
        todo!()
    }
}

fn main() {
    // Various ways of constructing operations
    let _get_pokemon_species = GetPokemonSpecies::from_handler(get_pokemon_species);
    let _empty_operation = EmptyOperation::from_handler(empty_operation);
    let _empty_operation = EmptyOperation::from_service(EmptyOperationServiceA);
    let empty_operation = EmptyOperation::from_service(EmptyOperationServiceB);
    let get_pokemon_species = GetPokemonSpecies::from_handler(get_pokemon_species_stateful);

    // We can apply a layer to them
    let get_pokemon_species = get_pokemon_species.layer(MapResponseLayer::new(|resp| resp));

    // We can build the `PokemonService` with static type checking
    let pokemon_service = PokemonService::builder()
        .get_pokemon_species(get_pokemon_species)
        .empty_operation(empty_operation)
        .build();

    // We can apply a layer to all routes
    let pokemon_service = pokemon_service.layer(MapResponseLayer::new(|resp| resp));

    let addr = "localhost:8000".parse().unwrap();
    hyper::server::Server::bind(&addr).serve(pokemon_service.into_make_service());
}

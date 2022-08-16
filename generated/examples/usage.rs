use std::{convert::Infallible, future::Ready, task::Poll};

use generated::{operations::*, services::*, structures::*};
use runtime::operation::{AdjoinState, OperationError, OperationShapeExt};
use tower::{util::MapResponseLayer, Service};

/// Fallible handler with state
async fn get_pokemon_species_stateful(
    _input: GetPokemonSpeciesInput,
    _state: usize,
) -> Result<GetPokemonSpeciesOutput, ResourceNotFoundException> {
    todo!()
}

/// Fallible handler without state
async fn get_pokemon_species(
    _input: GetPokemonSpeciesInput,
) -> Result<GetPokemonSpeciesOutput, ResourceNotFoundException> {
    todo!()
}

/// Infallible handler without state
async fn empty_operation(_input: EmptyOperationInput) -> EmptyOperationOutput {
    todo!()
}

/// Bespoke implementation of `EmptyOperation`.
#[derive(Clone)]
struct EmptyOperationService;

impl Service<EmptyOperationInput> for EmptyOperationService {
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

fn main() {
    // Various ways of constructing operations
    let _get_pokemon_species = GetPokemonSpecies::from_handler(get_pokemon_species);
    let _empty_operation = EmptyOperation::from_handler(empty_operation);
    let empty_operation = EmptyOperation::from_service(EmptyOperationService);
    let get_pokemon_species =
        GetPokemonSpecies::from_handler(get_pokemon_species_stateful.with_state(29));

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

use std::task::{Context, Poll};

use futures::{future::Map, FutureExt};
use http_body::combinators::BoxBody;
use hyper::{body::Bytes, Error};
use runtime::{
    make_service::IntoMakeService,
    operation::{
        IntoResponse, Operation, OperationNotSet, OperationShape, UpgradeLayer, UpgradedService,
    },
    protocols::AWSRestJsonV1,
    router::{
        rest::{self, RoutingFuture},
        RouteService,
    },
    service::ServiceError,
};
use tower::{util::BoxCloneService, Layer, Service, ServiceExt};

use crate::operations::{get_pokemon_species::GetPokemonSpecies, EmptyOperation};

#[derive(Clone)]
pub struct PokemonService<S> {
    router: rest::Router<S>,
}

impl<S> PokemonService<S> {
    /// Apply a [`Layer`] uniformly across all routes.
    pub fn layer<L>(self, layer: L) -> PokemonService<L::Service>
    where
        L: Layer<S>,
    {
        PokemonService {
            router: self.router.layer(layer),
        }
    }

    /// Converts [`PokemonService`] into a [`IntoMakeService`].
    pub fn into_make_service(self) -> IntoMakeService<Self> {
        IntoMakeService::new(self)
    }
}

impl PokemonService<()> {
    /// Creates a empty [`PokemonServiceBuilder`].
    pub fn builder() -> PokemonServiceBuilder<OperationNotSet, OperationNotSet> {
        PokemonServiceBuilder {
            get_pokemon_species: OperationNotSet,
            empty_operation: OperationNotSet,
        }
    }
}

impl<B, S> Service<http::Request<B>> for PokemonService<S>
where
    S: Service<http::Request<B>, Response = http::Response<BoxBody<Bytes, Error>>>,
    S: Clone,
{
    type Response = http::Response<BoxBody<Bytes, Error>>;

    type Error = S::Error;

    type Future = Map<
        RoutingFuture<S, http::Request<B>>,
        fn(
            Result<Self::Response, ServiceError<rest::RoutingError, S::Error>>,
        ) -> Result<Self::Response, S::Error>,
    >;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.router.poll_ready(cx).map_err(|err| match err {
            ServiceError::Routing(_) => unreachable!("routing errors cannot occur during poll"),
            ServiceError::Poll(err) => err,
        })
    }

    fn call(&mut self, req: http::Request<B>) -> Self::Future {
        self.router.call(req).map(|result| match result {
            Ok(ok) => Ok(ok),
            Err(ServiceError::Poll(err)) => Err(err),
            Err(ServiceError::Routing(err)) => Ok(err.into_response()),
        })
    }
}

/// The [`PokemonService`] builder.
pub struct PokemonServiceBuilder<Op1, Op2> {
    get_pokemon_species: Op1,
    empty_operation: Op2,
}

impl<Op1, Op2> PokemonServiceBuilder<Op1, Op2> {
    pub fn get_pokemon_species<S, L>(
        self,
        operation: Operation<S, L>,
    ) -> PokemonServiceBuilder<Operation<S, L>, Op2> {
        PokemonServiceBuilder {
            get_pokemon_species: operation,
            empty_operation: self.empty_operation,
        }
    }

    pub fn empty_operation<S, L>(
        self,
        operation: Operation<S, L>,
    ) -> PokemonServiceBuilder<Op1, Operation<S, L>> {
        PokemonServiceBuilder {
            get_pokemon_species: self.get_pokemon_species,
            empty_operation: operation,
        }
    }
}

impl<S1, S2, L1, L2> PokemonServiceBuilder<Operation<S1, L1>, Operation<S2, L2>> {
    pub fn build<B, Exts1, Exts2>(self) -> PokemonService<RouteService<B>>
    where
        // GetPokemonSpecies composition
        UpgradeLayer<AWSRestJsonV1, GetPokemonSpecies, Exts1, B>: Layer<S1>,
        L1: Layer<UpgradedService<AWSRestJsonV1, GetPokemonSpecies, Exts1, B, S1>>,
        S1: Service<(<GetPokemonSpecies as OperationShape>::Input, Exts1)>,
        L1::Service: Service<http::Request<B>, Response = http::Response<BoxBody<Bytes, Error>>>,
        L1::Service: Clone + Send + 'static,
        <L1::Service as Service<http::Request<B>>>::Future: Send + 'static,
        <L1::Service as Service<http::Request<B>>>::Error:
            Into<Box<dyn std::error::Error + Send + Sync>> + 'static,

        // EmptyOperation composition
        UpgradeLayer<AWSRestJsonV1, EmptyOperation, Exts2, B>: Layer<S2>,
        L2: Layer<UpgradedService<AWSRestJsonV1, EmptyOperation, Exts2, B, S2>>,
        S2: Service<(<EmptyOperation as OperationShape>::Input, Exts2)>,
        L2::Service: Service<http::Request<B>, Response = http::Response<BoxBody<Bytes, Error>>>,
        L2::Service: Clone + Send + 'static,
        <L2::Service as Service<http::Request<B>>>::Future: Send + 'static,
        <L2::Service as Service<http::Request<B>>>::Error:
            Into<Box<dyn std::error::Error + Send + Sync>> + 'static,
    {
        PokemonService {
            router: [
                (
                    GetPokemonSpecies::NAME,
                    BoxCloneService::new(self.get_pokemon_species.upgrade().map_err(Into::into)),
                ),
                (
                    EmptyOperation::NAME,
                    BoxCloneService::new(self.empty_operation.upgrade().map_err(Into::into)),
                ),
            ]
            .into_iter()
            .collect(),
        }
    }
}

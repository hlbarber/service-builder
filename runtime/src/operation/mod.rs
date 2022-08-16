mod handler;
mod shape;
mod upgrade;

use tower::{
    layer::util::{Identity, Stack},
    Layer,
};

pub use handler::*;
pub use shape::*;
pub use upgrade::*;

/// Represents a Smithy operation, coupled with model [`Layer`] and HTTP [`Layer`].
pub struct Operation<S, L = Identity> {
    inner: S,
    layer: L,
}

impl<S, L> Operation<S, L> {
    /// Takes the [`Operation`], which contains the inner [`Service`](tower::Service), the HTTP [`Layer`] `L` and
    /// composes them together using [`UpgradeLayer`] for a specific protocol and [`OperationShape`].
    ///
    /// The composition is made explicit in the method constraints and return type.
    pub fn upgrade<P, Op, B>(self) -> <Stack<UpgradeLayer<P, Op, B>, L> as Layer<S>>::Service
    where
        UpgradeLayer<P, Op, B>: Layer<S>,
        L: Layer<<UpgradeLayer<P, Op, B> as Layer<S>>::Service>,
    {
        let Self { inner, layer } = self;
        let layer = Stack::new(UpgradeLayer::new(), layer);
        layer.layer(inner)
    }
}

impl<S> Operation<S> {
    /// Creates an [`Operation`] from a [`Service`](tower::Service).
    pub fn from_service(inner: S) -> Self {
        Self {
            inner,
            layer: Identity::new(),
        }
    }
}

impl<Output, Error, H> Operation<IntoService<Output, Error, H>> {
    /// Creates an [`Operation`] from a [`Handler`].
    pub fn from_handler<Input>(handler: H) -> Self
    where
        H: Handler<Input, Output, Error>,
    {
        Self {
            inner: handler.into_service(),
            layer: Identity::new(),
        }
    }
}

impl<S, L> Operation<S, L> {
    /// Applies a [`Layer`] to the operation after is has been upgraded to a [`Service`] accepting
    /// and returning [`http`] types.
    pub fn layer<NewL>(self, layer: NewL) -> Operation<S, Stack<L, NewL>> {
        Operation {
            inner: self.inner,
            layer: Stack::new(self.layer, layer),
        }
    }
}

/// A marker struct indicating an [`Operation`] has not been set in a builder.
pub struct OperationNotSet;

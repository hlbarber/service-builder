mod flattened;
mod handler;
mod http_conversions;
mod shape;
mod upgrade;

use tower::{
    layer::util::{Identity, Stack},
    Layer,
};

pub use flattened::*;
pub use handler::*;
pub use http_conversions::*;
pub use shape::*;
pub use upgrade::*;

/// Represents a Smithy operation, coupled with model [`Layer`] and HTTP [`Layer`].
pub struct Operation<S, L = Identity> {
    inner: S,
    layer: L,
}

type StackedUpgradeService<P, Op, E, B, L, S> =
    <Stack<UpgradeLayer<P, Op, E, B>, L> as Layer<S>>::Service;

impl<S, L> Operation<S, L> {
    /// Takes the [`Operation`], which contains the inner [`Service`](tower::Service), the HTTP [`Layer`] `L` and
    /// composes them together using [`UpgradeLayer`] for a specific protocol and [`OperationShape`].
    ///
    /// The composition is made explicit in the method constraints and return type.
    pub fn upgrade<P, Op, E, B>(self) -> StackedUpgradeService<P, Op, E, B, L, S>
    where
        UpgradeLayer<P, Op, E, B>: Layer<S>,
        L: Layer<<UpgradeLayer<P, Op, E, B> as Layer<S>>::Service>,
    {
        let Self { inner, layer } = self;
        let layer = Stack::new(UpgradeLayer::new(), layer);
        layer.layer(inner)
    }
}

impl<Op, S, PollError> Operation<IntoUnflattened<Op, S, PollError>> {
    /// Creates an [`Operation`] from a [`Service`](tower::Service).
    pub fn from_service<Exts>(inner: S) -> Self
    where
        Op: OperationShape,
        S: Flattened<Op, Exts, PollError>,
    {
        Self {
            inner: inner.into_unflatten(),
            layer: Identity::new(),
        }
    }
}

impl<Op, H> Operation<IntoService<Op, H>> {
    /// Creates an [`Operation`] from a [`Handler`].
    pub fn from_handler<Exts>(handler: H) -> Self
    where
        Op: OperationShape,
        H: Handler<Op, Exts>,
    {
        Self {
            inner: handler.into_service(),
            layer: Identity::new(),
        }
    }
}

impl<S, L> Operation<S, L> {
    /// Applies a [`Layer`] to the operation _after_ it has been upgraded via [`Operation::upgrade`].
    pub fn layer<NewL>(self, layer: NewL) -> Operation<S, Stack<L, NewL>> {
        Operation {
            inner: self.inner,
            layer: Stack::new(self.layer, layer),
        }
    }
}

/// A marker struct indicating an [`Operation`] has not been set in a builder.
pub struct OperationNotSet;

/// The operation [`Service`] has two classes of failure modes - the failure models specified by
/// the Smithy model and failures to [`Service::poll_ready`].
pub enum OperationError<PollError, SmithyError> {
    /// A [`Service::poll_ready`] failure occurred.
    Poll(PollError),
    /// An error modelled by the Smithy model occurred.
    Smithy(SmithyError),
}

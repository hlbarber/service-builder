use super::{Flattened, Handler, IntoService, IntoUnflattened, Operation};

/// Mirrors the Smithy Operation shape.
pub trait OperationShape {
    const NAME: &'static str;

    /// The operation input.
    type Input;
    /// The operation output.
    type Output;
    /// The operation error.
    type Error;
}

/// An extension trait over [`OperationShape`].
pub trait OperationShapeExt: OperationShape {
    /// Creates a new [`Operation`] for well-formed [`Handler`]s.
    fn from_handler<H, Exts>(handler: H) -> Operation<IntoService<Self, H>>
    where
        H: Handler<Self, Exts>,
        Self: Sized,
    {
        Operation::from_handler(handler)
    }

    /// Creates a new [`Operation`] for well-formed [`Service`](tower::Service)s.
    fn from_service<S, Exts, PollError>(svc: S) -> Operation<IntoUnflattened<Self, S, PollError>>
    where
        S: Flattened<Self, Exts, PollError>,
        Self: Sized,
    {
        Operation::from_service(svc)
    }
}

impl<S> OperationShapeExt for S where S: OperationShape {}

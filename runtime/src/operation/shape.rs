use tower::Service;

use super::{Handler, HandlerExt, IntoService, Operation, OperationError};

/// Mirrors the Smithy Operation shape.
pub trait OperationShape {
    const NAME: &'static str;

    type Input;
    type Output;
    type Error;
}

/// An extension trait over [`OperationShape`].
pub trait OperationShapeExt: OperationShape {
    /// Creates a new [`Operation`] for well-formed [`Handler`]s.
    fn from_handler<H>(
        handler: H,
    ) -> Operation<IntoService<<Self as OperationShape>::Output, <Self as OperationShape>::Error, H>>
    where
        H: Handler<
            <Self as OperationShape>::Input,
            <Self as OperationShape>::Output,
            <Self as OperationShape>::Error,
        >,
    {
        Operation::from_service(handler.into_service())
    }

    /// Creates a new [`Operation`] for well-formed [`Service`]s.
    fn from_service<S, PollError>(svc: S) -> Operation<S>
    where
        S: Service<
            <Self as OperationShape>::Input,
            Response = <Self as OperationShape>::Output,
            Error = OperationError<PollError, <Self as OperationShape>::Error>,
        >,
    {
        Operation::from_service(svc)
    }
}

impl<S> OperationShapeExt for S where S: OperationShape {}

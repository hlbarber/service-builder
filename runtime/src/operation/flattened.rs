use std::{
    marker::PhantomData,
    task::{Context, Poll},
};

use tower::Service;

use super::{OperationError, OperationShape};

/// A utility trait used to provide an even interface for all operation services.
///
/// This serves to take [`Service`]s of the form `Service<(Input, Arg0, Arg1, ...)>` to the canonical representation of
/// `Service<(Input, (Arg0, Arg1, ...))>` inline with [`IntoService`](super::IntoService).
pub trait Flattened<Op, Exts, PollError>:
    Service<Self::Flattened, Response = Op::Output, Error = OperationError<PollError, Op::Error>>
where
    Op: OperationShape,
{
    type Flattened;

    // Unflatten the request type.
    fn unflatten(input: Op::Input, exts: Exts) -> Self::Flattened;
}

// `Service<Op::Input>`
impl<Op, S, PollError> Flattened<Op, (), PollError> for S
where
    Op: OperationShape,
    S: Service<Op::Input, Response = Op::Output, Error = OperationError<PollError, Op::Error>>,
{
    type Flattened = Op::Input;

    fn unflatten(input: Op::Input, _exts: ()) -> Self::Flattened {
        input
    }
}

// `Service<(Op::Input, Arg0)>`
impl<Op, Arg0, S, PollError> Flattened<Op, (Arg0,), PollError> for S
where
    Op: OperationShape,
    S: Service<
        (Op::Input, Arg0),
        Response = Op::Output,
        Error = OperationError<PollError, Op::Error>,
    >,
{
    type Flattened = (Op::Input, Arg0);

    fn unflatten(input: Op::Input, exts: (Arg0,)) -> Self::Flattened {
        (input, exts.0)
    }
}

// `Service<(Op::Input, Arg0, Arg1)>`
impl<Op, Arg0, Arg1, S, PollError> Flattened<Op, (Arg0, Arg1), PollError> for S
where
    Op: OperationShape,
    S: Service<
        (Op::Input, Arg0, Arg1),
        Response = Op::Output,
        Error = OperationError<PollError, Op::Error>,
    >,
{
    type Flattened = (Op::Input, Arg0, Arg1);

    fn unflatten(input: Op::Input, exts: (Arg0, Arg1)) -> Self::Flattened {
        (input, exts.0, exts.1)
    }
}

/// An extension trait of [`Flattened`].
pub trait FlattenedExt<Op, Exts, PollError>: Flattened<Op, Exts, PollError>
where
    Op: OperationShape,
{
    /// Convert the [`Flattened`] into a canonicalized [`Service`].
    fn into_unflatten(self) -> IntoUnflattened<Op, Self, PollError>
    where
        Self: Sized,
    {
        IntoUnflattened {
            inner: self,
            _operation: PhantomData,
            _poll_error: PhantomData,
        }
    }
}

impl<F, Op, Exts, PollError> FlattenedExt<Op, Exts, PollError> for F
where
    Op: OperationShape,
    F: Flattened<Op, Exts, PollError>,
{
}

/// A [`Service`] canonicalizing the request type of a [`Flattened`].
#[derive(Debug)]
pub struct IntoUnflattened<Op, S, PollError> {
    inner: S,
    _operation: PhantomData<Op>,
    _poll_error: PhantomData<PollError>,
}

impl<Op, S, PollError> Clone for IntoUnflattened<Op, S, PollError>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _operation: PhantomData,
            _poll_error: PhantomData,
        }
    }
}

impl<Op, S, Exts, PollError> Service<(Op::Input, Exts)> for IntoUnflattened<Op, S, PollError>
where
    Op: OperationShape,
    S: Flattened<Op, Exts, PollError>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = <S as Service<S::Flattened>>::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, (input, exts): (Op::Input, Exts)) -> Self::Future {
        let req = S::unflatten(input, exts);
        self.inner.call(req)
    }
}

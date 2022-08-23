# Schematic

## Existing

```
Outer layers {customer}
|- Router {runtime}
|-- Operation agnostic layers {customer via Router::layer} 
|--- Upgrade layer - converts between Smithy types and http {runtime + codegen}
|---- Handler fn {customer}
```

## Proposal

The trait `OperationShape` models Smithy operations.

```rust
trait OperationShape {
    type Input;
    type Output;
    type Error;
}
```

A service builder sets an operation by accepting `Operation<S, L>` where `S: Service<(Op::Input, Exts), Response = Op::Output, Error = PollError | Op::Error>>`, `OperationInput: FromRequest<P, Op>`, `Exts: FromRequest<P, Op>`, `Op::Output: IntoResponse<P, Op>`, and `Op::Error: IntoResponse<P, Op>` for `Op: OperationShape`.

A `Operation` includes two constructors, `from_handler` which accepts a `H: Handler<Op, Exts>` and `from_service` which accepts a `S: Flattened<Op, Exts, PollError>`. The trait `Handler<Op, Ext>` is enjoyed by all closures which accept `(Op::Input, ...)` and return `Result<Op::Input, Op::Error>`. The trait `Flattened<Op, Exts, PollError>` is enjoyed by all `Service<(Op::Input, ...), Response = Op::Output, Error = PollError | Op::Error>`. Both `Handler` and `Flattened` work to provide a common interface to convert to `S: Service<(Op::Input, Exts), Response = Op::Output, Error = PollError | Op::Error>` in `Operation<S, L>`.

The `UpgradeLayer<P, Op, Exts, B>` is a `Layer<S>`, applied to such `S`. It uses the `FromRequest<P, Op>` and `IntoResponse<P, Op>` to wrap `S` in middleware - converting `S: Service<(Op::Input, Exts), Response = Op::Output, Error = PollError | Op::Error>` to `S: Service<http::Request, Response = http::Response, Error = PollError>` in a protocol and operation aware way.

The `Operation<S, L>::upgrade<P, Op, Exts, B>` takes `S`, applies `UpgradeLayer<P, Op, Exts, B>`, then applies the `L: Layer<UpgradeLayer::Service>`. The `L` in `Operation<S, L>` can be set by the user to provide operation specific HTTP middleware. The `Operation::upgrade` is called in the service builder `build` method and the composition is immediately `Box`'d and collected up into the protocol specific router alongside the other routes.

In this way the customer can provide, for a specific operation, middleware around `S` _and_ `S` after it's upgraded to a HTTP service via `L`.

```
Outer layers {customer}
|- Router {runtime + codegen}
|-- Operation agnostic layers {customer via Router::layer} 
|--- Operation aware first party layers - logging/auth {runtime + codegen}
|---- Operation aware third party layers {customer via Operation::layer}
|----- Upgrade layer - converts between Smithy types and http {runtime + codegen}
|------ Handler service {customer}
```

# Reference Model

```smithy
$version: "1.0"

namespace com.aws.example

use aws.protocols#restJson1

@restJson1
service PokemonService {
    operations: [GetPokemonSpecies, EmptyOperation],
}

/// Retrieve information about a Pokémon species.
@readonly
@http(uri: "/pokemon-species/{name}", method: "GET")
operation GetPokemonSpecies {
    input: GetPokemonSpeciesInput,
    output: GetPokemonSpeciesOutput,
    errors: [ResourceNotFoundException],
}

@input
structure GetPokemonSpeciesInput {
    @required
    @httpLabel
    name: String
}

structure GetPokemonSpeciesOutput {
    @required
    name: String,
}

@readonly
@http(uri: "/empty-operation", method: "GET")
operation EmptyOperation {
    input: EmptyOperationInput,
    output: EmptyOperationOutput,
}

@input
structure EmptyOperationInput { }

@output
structure EmptyOperationOutput { }
```

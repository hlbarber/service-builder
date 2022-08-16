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

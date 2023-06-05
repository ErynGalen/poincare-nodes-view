# Poincare Nodes View
This programs pretty-prints the XML coming from poincare logs.
This logs are generated using [the instrument-poincare branch from my fork of Upsilon](https://github.com/ErynGalen/Upsilon/tree/instrument-poincare).

## Usage
### Requirements
* a Rust toolchain
### Running
To run this program you can use `cargo run`. This will run a debug build.

The program reads a the file `poincare-log.xml`, expected to be in the directory from which you are running the program in.

It outputs the parsed and beautified log of actions preformed by Poincare when simplifying an expression.
See [the parsed action tree](#parsed-action-tree) for more information.

### Building
To compile it in release mode you can use `cargo build --release`.
The resulting binary will be `target/release/poincare-nodes-view`.

## XML Log Format
At the top-level of the XML file there should only be `ReduceProcess` nodes.
A `ReduceProcess` node is made of:
* a `OriginalExpression` and a `ResultExpression` containing [`PoincareNode`s](#poincare-node), which represent the expression before and after the simplification
* several `Step` nodes containing two [`PoincareNode`s](#poincare-node), representing some part of the expression before and after the simplification step.
### Poincare Node
A Poincare node has the following form:
`<NodeName id="..." attr1="..." attr2="..."> ...children... </NodeName>`.
It can have 0 or more children, which are others Poincare nodes.

Each node has a unique `id` which is preserved across simplification steps and simplifications.

## Parsed Action Tree
TODO: explain the output of the program.

# Poincare Nodes View
`poincare-nodes-view` pretty-prints the XML coming from poincare logs.
This logs are generated using [the instrument-poincare branch from my fork of Upsilon](https://github.com/ErynGalen/Upsilon/tree/instrument-poincare).

## Compiling Upsilon to get the logs
You can get [my fork of Upsilon](https://github.com/ErynGalen/Upsilon), and `git switch instrument-poincare` to have Upsilon with the Poincare logs.
The compilation can then be done (only debug mode, i.e. `make DEBUG=1 ...`, has been tested).

The simulator should be used to obtain the logs, by simply doing some calculation. Every time Poincare is used its actions are logged.
So that the logging actually happens, the branch defines `POINCARE_TREE_LOG=1` for the simulator.

## Usage
### Requirements
* a Rust toolchain
### Running
To run `poincare-nodes-view` you can use `cargo run`. This will run a debug build.

`poincare-nodes-view` reads the file `poincare-log.xml`, expected to be in the directory from which you are running the program in.
See [the format of the file](#xml-log-format) for more information.

`poincare-nodes-view` outputs in a (hopefully) readable form the log of actions preformed by Poincare when simplifying an expression.
See [the parsed action tree](#parsed-action-tree) for more information.

### Building
To compile it in release mode you can use `cargo build --release`.
The resulting binary will be `target/release/poincare-nodes-view`.

## XML Log Format
At the top-level of the XML file there should only be `ReduceProcess` nodes.
A `ReduceProcess` node is made of:
* a `OriginalExpression` and a `ResultExpression` containing [`PoincareNode`s](#poincare-node), which represent the expression before and after the simplification
* several `Step` nodes containing two [`PoincareNode`s](#poincare-node), representing **some part** of the expression before and after the simplification step. The unique ids in the expression before the step can be used to know which part of the expression is simplified by the step.
### Poincare Node
A Poincare node has the following form:
`<NodeName id="..." attr1="..." attr2="..."> ...children... </NodeName>`.
It can have 0 or more children, which are others Poincare nodes.

Each node has a unique `id` which is preserved across simplification steps and simplifications.

## Parsed Action Tree
### ReduceAction
Each Reduce action has the form:
```
* Reduce <original poincare expression>:
    <step1>
    <step2>
    ...
*-> <result poincare expression>
```
There may be 0 or more steps.

### Step
Each simplification step has the form:
```
/> <name>
| <poincare expression before the step>
|    <substep1>
|    <substep2>
|    ...
\_ <poincare expression>
```
Each step has a name which represents which function is achieving it.

There may be 0 or more substeps.

### Poincare expression
A Poincare expression has the form:
```
<node name>(<unique id>): <node representation> { <child node 1>, <child node 2>, ... }
```
There may or may not be a [node reprensentation](#some-node-representations), which is usually the value or name of the node, but may be anything else representing the node.

A node can have 0 or more children.
#### Some node representations
Here are explanations about the node representations:
* BasedInteger: `<value>__<base>`
* CodePointLayout: `<code point>`
* Decimal: `<sign><mantissa>x10^<exponent>`
* Integer: `<value>`
* Matrix: `rows: <number of rows?>, columns: <number of columns?>`
* Rational: `<sign><numerator>/<denominator>`
* SymbolAbstract / Symbol / Sequence / Function / Constant: `<name of the symbol>`
* Unit: `<prefix><root symbol>`

These representations are based off of the attributes logged by `logAttributes()` in Poincare.

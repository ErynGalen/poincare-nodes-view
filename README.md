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

By default, `poincare-nodes-view` reads the file `poincare-log.xml`, expected to be in the directory from which you are running the program in. If you wish to read other files, you can pass file names as options, e.g. `cargo run -- file-to-read.xml`. See [command line options](#command-line-options) for more information about how to pass options.
See [the format of the file](#xml-log-format) for more information.

`poincare-nodes-view` outputs in a (hopefully) readable form the log of actions preformed by Poincare when simplifying an expression.
See [the parsed action tree](#parsed-action-tree) for more information.

### Command line options
When running with `cargo run`, command line option must be passed as follow: `cargo run -- --opt1 --opt2 file1 file2`.
You can specify files and options in any order.

All the files passed to `poincare-nodes-view` will be read in the order they're supplied in the command line.

By default intermediate states in steps are displayed, if you want to hide them, you can use:
* `--no-states`

By default some reduction steps aren't displayed, the following options are available to show them:
* `--useless`: show all the steps, even those doing nothing. Implies all the following options
* `--number-to-rational`: show steps which transform f.e. a BasedInteger into a Rational with the same value
* `--to-undef`: show steps leading to `Undefined` node

By default the nodes are displayed in a short form representing them briefly. If you wish to display the [long form](#poincare-expression), you can use:
* `--long`

### Building
To compile it in release mode you can use `cargo build --release`.
The resulting binary will be `target/release/poincare-nodes-view`.

## XML Log Format
At the top-level of the XML file there should only be `Step` nodes.
A `Step` node is made of:
* other `Step` nodes, which are substeps
* `State` nodes, which contain a [`PoincareNode`](#poincare-node) representing the state. A `State` node may have a `name` attribute to 

A state represent **some part** of the expression being reduced. The unique ids in the expression's long form  can be used to distinguish which part of the expression is simplified by the step.

A step should contain a `State` whose name is `before` and a `State` whose name is `after`. These states should represent the expression before and after the simplification step.
### Poincare Node
A Poincare node has the following form:
`<NodeName id="..." attr1="..." attr2="..."> ...children... </NodeName>`.
It can have 0 or more children, which are others Poincare nodes.

Each node has a unique `id` which is preserved across simplification steps and simplifications.

## Parsed Action Tree
### Step
Each simplification step has the form:
```
/> <name>
| <poincare expression before the step>
|    <substep1>
|    <substep2>
|- <state1>
|    <substep3>
|- <state2>
|    ...
\_ <poincare expression>
```
Each step has a name which represents which function is achieving it.

There may be 0 or more substeps, and 0 or more states. Except for `before` and `after` states, the order of substeps and states is the order in which actions and states occur.

A state is either a Poincare expression, or has the form `<name>: <poincare expression>` for named states.

### Poincare expression
There are two possible ways of displaying Poincare expression: long form and short form.
The short form tries to be as close as possible to mathematical expressions, whereas the long form tries to give as much information as possible.

The following describes the **long form**.

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

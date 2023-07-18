# Cargo BSP

An implementation of the [Build Server Protocol](https://github.com/build-server-protocol/build-server-protocol) for Cargo.

## Status

Currently, the server handles the most crucial requests and is functional with the [Build Server Protocol plugin](https://lp.jetbrains.com/new-bazel-plugin/#install) in IntelliJ IDEA.

The server supports the following actions:

- importing a project's targets and sources,
- compiling a project,
- running a project,
- testing a project,
- reloading a project.

## Repository structure

The repository is split into two crates: ```bsp-types``` and ```cargo-bsp```.

The ```bsp-types``` crate contains all BSP structures specified in [BSP specification](https://build-server-protocol.github.io/docs/specification) rewritten into Rust.

The ```cargo-bsp``` crate contains the implementation of the Cargo BSP server itself.

## Installation

1. Have [Rust toolchain](https://rustup.rs) installed
2. Clone this repository and run in its directory:

```shell
cargo build
```

3. In the directory of the desired Rust project run this script (fill in the SERVER_PATH variable with the directory to the cloned repository):

```shell
SERVER_PATH="" 
mkdir .bsp
cat >> .bsp/cargo-bsp.json << EOF
{
  "name": "cargobsp",
  "argv": [
    "$SERVER_PATH/target/debug/server"
  ],
  "version": "0.1.0",
  "bspVersion": "2.1.0-M4",
  "languages": [
    "rust"
  ]
}
```

4. Open the Rust project in IntelliJ IDEA with enabled [Build Server Protocol plugin](https://lp.jetbrains.com/new-bazel-plugin/#install) and disabled Rust plugin (if installed)

## Tests

To run all tests in the project, run ```cargo test```

### Integration tests

```crates/tests``` directory contains integration tests checking the client-server communication. These tests can be run with:

```cargo test --test integration_test```

### Unit tests

Some files also contain unit tests. These tests can be run with:

- ```cargo test --lib``` - to run all unit tests
- ```cargo test --lib --package <package_name>``` - to run tests of a library target for specific package

## Troubleshooting

The server's logs can be find in the ```.cargobsp``` directory in the Rust project's directory.

## Future work

The server provides the most crucial functionalities. However there are still some areas to improve and future work to be done:

1. The server cannot run unit tests from the project separately. However some research has already been made regarding that topic, see [file](crates/cargo-bsp/src/project_model/_unit_tests_discovery.rs)
2. Unimplemented requests: resources, debug, clean cache, dependency modules, dependency sources, inverse sources, output paths
3. Canceling request does not cancel all started tasks, only the root tasks
4. The server does not track the changes in files and build targets and only checks their state after receiving reload request.

## Authors

- Katarzyna Kloc - [@KatKlo](https://github.com/KatKlo)
- Patryk Bundyra - [@PBundyra](https://github.com/PBundyra)
- Julia Podrażka - [@julia-podrazka](https://github.com/julia-podrazka)
- Tomasz Głąb - [@Toomimi](https://github.com/Toomimi)
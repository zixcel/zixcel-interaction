# zixcel-interaction

Transport-neutral resource and action contracts for applications that validate structured values and handle explicit success, conflict and rejection outcomes. Use this crate to share interaction shapes between an owner service and its clients.

## Install

```toml
[dependencies]
zixcel-interaction = "0.10.0"
```

This dependency uses crates.io; no private registry is required.

## Capabilities

- Validate bounded value schemas and resource snapshots.
- Describe operations, input declarations and resource revisions.
- Distinguish structural validation from operation execution.

## Example

```rust
use zixcel_interaction::ValueSchema;

assert!(ValueSchema::Boolean.is_valid());
```

## Features and requirements

Requires Rust 1.97 or newer. The crate is a reusable library, not a command-line application.

## Boundaries

Applications implement authorization, transport, persistence, concurrency control and rendering. The Rust API and the separate JavaScript package have different exports; neither validator executes actions.

## Development

```sh
cargo fmt --all -- --check
cargo test --locked --all-features
cargo clippy --locked --all-targets --all-features -- -D warnings
```

## Documentation and license

[API documentation](https://docs.rs/zixcel-interaction) · [Source](https://github.com/zixcel/zixcel-interaction) · [Usage guide](https://github.com/zixcel/zixcel-interaction/blob/main/docs/getting-started.md)

Apache-2.0. Retain the package LICENSE and NOTICE; see the source repository for security reporting and contribution guidelines.

## JavaScript package

The separate `@zixcel/interaction` ESM package exposes JavaScript validators and TypeScript declarations. npm availability must be checked independently of this crate.

See the [JavaScript usage guide](docs/javascript.md) for examples and its distinct exports.

## TypeScript development

Shared wire contracts live in `web/types/`; executable guards are strict
TypeScript. Run `npm ci --ignore-scripts`, `npm run typecheck`, and `npm test`.
The build produces browser-compatible ESM and matching declarations in `dist/`.
Package exports resolve both from the same build. Inputs remain `unknown` until
runtime validation succeeds. Shape validation grants no execution authority.

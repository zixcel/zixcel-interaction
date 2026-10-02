# @zixcel/interaction

Describe resource reads, actions and state changes with typed requests and explicit results.

## What you can do

- Validate an interaction before dispatch.
- Distinguish accepted changes, conflicts and rejected requests.

## Current scope

The application supplies authorization and execution. A declared action is not permission to perform it.

Package distribution is not activated by this documentation. Use the checked-in source and the declared dependency versions; published availability must be verified separately.

## Getting started

Install Rust 1.97 or newer and make the declared dependencies available. Use the configured private registry when a dependency is not distributed publicly. Run from this repository:

```sh
npm install
npm run test
```

## Documentation and source

[Usage guide](docs/getting-started.md)

[Implementation and public interfaces](src) · [Verification cases](tests) · [Verification cases](test) · [Contributing](CONTRIBUTING.md) · [Security reporting](SECURITY.md) · [License](LICENSE) · [Attribution notices](NOTICE)

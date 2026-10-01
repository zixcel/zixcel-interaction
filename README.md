# zixcel-interaction

Version 0.10.0. Small UI-, product- and transport-independent Rust contracts for
reading resources, invoking typed actions and subscribing after a snapshot cursor.
Dependencies: serde and serde_json. No Nuxt, Hatter, sem-lang, HTTP or async runtime.

`ValueSchema` is a closed, bounded value algebra (not an incomplete JSON Schema
implementation). Object fields are closed, required means presence rather than
truthiness, and diagnostics use JSON pointers. Unknown wire fields are rejected.

`Outcome<T>` represents Success, ValidationError, Conflict, Forbidden, Unavailable,
NotFound and PreconditionFailed independently of HTTP status. Resource, contract
and change revisions are opaque equality tokens, not sortable timestamps.

The handler owns authenticated identity, access grants, atomic compare-and-swap,
durable idempotency and consistent snapshot/cursor capture. Authorize before
looking up resources or replaying receipts. A request reference must be scoped to
that identity and bound to the complete request: changed payload under the same
reference is a conflict. This package does not claim to implement those policies.

An expired stream cursor produces `Reset`, requiring a complete coherent read.
No partial change or browser-inferred HTTP meaning is a successful read. Values
of unreadable resources must not be serialized. Connection protection and limits
belong to the transport; encode/decode application outcomes without changing them.

Consumers use a published 0.10.0 archive from zixcel-private, never a source path.

## Package integration

The package is an independently consumable unit. Callers reference its documented
interface through a versioned dependency and own application-specific composition
and integration.

## Distribution license

Apache-2.0. Copyright 2026 HAT Inc. See [LICENSE](LICENSE) and [NOTICE](NOTICE). Earlier license files and third-party terms remain applicable to their respective portions.

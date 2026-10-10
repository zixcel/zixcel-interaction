# Validate data shapes and operation results

Validate input data before executing an operation. Distinguish structural validity from operation success. The [JavaScript guide](javascript.md) uses synthetic settings data and explains the contracts.

## Declare operation inputs

Import `/input` to combine input schemas with display labels. Validation does not coerce values: boolean `false` and string `"false"` remain distinct.

```js
import { validateInputDeclaration, validateInputValues } from '@zixcel/interaction/input'

const declaration = {
  action: {
    operation_id: 'example:update', target: 'example:settings', contract_revision: 'c1',
    availability: { state: 'available' }, expected_revision_required: true,
    input_schema: { type: 'object', fields: { enabled: { type: 'boolean' } }, required: ['enabled'] }
  },
  fields: { enabled: { label: 'Enabled', sensitive: false } }
}
console.log(validateInputDeclaration(declaration)) // []
console.log(validateInputValues(declaration, { enabled: false })) // []
console.log(validateInputValues(declaration, { enabled: 'false' })) // ['input/invalid']
```

Handle validation errors before dispatching an operation. The caller owns authorization, transport, mutation, concurrency control, atomicity, and persistence. Validation never grants permission or executes an operation.

## Rust API

The `zixcel-interaction` crate provides typed contracts and validation functions. JavaScript `validateResource`, `validateValue`, and `validateOutcome` and Rust `validate_invoke` have different API coverage. JavaScript consumers do not require Rust.

Install `zixcel-interaction = "0.10.0"` from crates.io. The crate uses edition 2024 and requires Rust 1.97 or later. It has no package-specific optional or default features. See the [crate README](../README.crate.md) for a compiled example.

## Verification

Run `npm test` for JavaScript and `cargo test --locked` for Rust in a source checkout. A local TGZ test validates the packaged JavaScript interface; official registry installation is a separate verification step. See the [README](../README.md) for supported runtimes and package boundaries.

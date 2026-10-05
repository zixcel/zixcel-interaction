# @zixcel/interaction

[日本語](README.ja.md)

Check the shape of data received by an application and distinguish success, conflict and rejection. This library provides JavaScript validators, TypeScript declarations and Rust source types and validation functions. The JavaScript package has no external runtime dependencies.

## Install

```sh
npm install @zixcel/interaction@0.10.0
```

This is a release candidate. Availability of this version on npm has not yet been established.

## Example: a settings screen

A settings screen receives a resource containing a boolean. Validate the resource and the proposed value before passing them to application code. A boolean `false` is valid; the string `'false'` is a different type.

```js
import { validateResource, validateValue, validateOutcome } from '@zixcel/interaction'

const booleanSchema = { type: 'boolean' }
const snapshot = {
  contract: {
    resource_id: 'example:settings', contract_revision: 'c1',
    value_schema: booleanSchema, readable: true,
    availability: { state: 'available' }, operations: []
  },
  resource_revision: 'r1', value: false
}

console.log(validateResource(snapshot))
console.log(validateValue(booleanSchema, false))
console.log(validateValue(booleanSchema, 'false'))

const outcome = { status: 'Conflict', reason: 'example:revision-changed', issues: [] }
console.log(validateOutcome(outcome))
console.log(outcome.status)
```

Output:

```text
[]
true
false
true
Conflict
```

`validateResource` returns an empty issue list for this valid snapshot. `validateValue` returns a boolean. `validateOutcome` checks the result's format: `true` does **not** mean the operation succeeded. Inspect `outcome.status` before handling a result as success.

## API and limits

The root export provides `CONTRACT`, `reference`, `validSchema`, `validateValue`, `validateResource` and `validateOutcome`. The `/input` export provides `inputLimits`, `validateInputDeclaration` and `validateInputValues`. Both entry points provide TypeScript declarations and use ESM imports; CommonJS is not advertised.

The application supplies authorization, communication, updates, concurrency control, atomicity and persistence. Validation neither grants permission nor executes an action. The JavaScript API does not provide the Rust `validate_invoke` function.

See the [usage guide](https://github.com/zixcel/zixcel-interaction/blob/main/docs/getting-started.md) for input declarations and the separate Rust source API. Version 0.x does not promise a stable 1.x compatibility policy.

## Verification

Run `npm test` from a source checkout. The example and installed archive consumer have been checked on Node 24.15.0 with npm 11.12.1; the source validators have also been checked on Node 24.13.1. These are tested environments, not a minimum runtime requirement. No Node engines restriction is declared. Tests are not included in the npm distribution.

[Source](https://github.com/zixcel/zixcel-interaction), [tests](https://github.com/zixcel/zixcel-interaction/tree/main/test), [security reporting](https://github.com/zixcel/zixcel-interaction/blob/main/SECURITY.md), [npm package](https://www.npmjs.com/package/@zixcel/interaction).

Licensed under [Apache-2.0](./LICENSE); retain [attribution notices](./NOTICE).

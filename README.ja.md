# @zixcel/interaction

[English](README.md)

利用アプリが受け取るデータの形を検証し、成功、競合、拒否を区別します。JavaScriptのvalidator、TypeScriptの型宣言、Rustのソース型と検証関数を提供します。JavaScript packageに外部runtime依存はありません。

## 導入

```sh
npm install @zixcel/interaction@0.10.0
```

これはリリース候補です。この版のnpm配布はまだ確認されていません。

## 例: 設定画面

booleanを含むresourceと入力値を、アプリ処理へ渡す前に検証します。booleanの`false`は正しい値ですが、stringの`'false'`は別の型です。

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

出力:

```text
[]
true
false
true
Conflict
```

`validateResource`はこの正しいsnapshotに対して空のissue一覧を返します。`validateValue`はbooleanを返します。`validateOutcome`は結果の形式を検証するため、`true`は操作成功を意味しません。成功として扱う前に`outcome.status`を確認してください。

## APIと制約

root exportは`CONTRACT`、`reference`、`validSchema`、`validateValue`、`validateResource`、`validateOutcome`です。`/input`は`inputLimits`、`validateInputDeclaration`、`validateInputValues`を公開します。両入口に型宣言があり、ESM importを使います。CommonJS対応は案内していません。
認可、通信、更新、同時実行制御、原子性、永続保存は利用アプリが担当します。検証は権限を与えず、actionも実行しません。JavaScript APIはRustの`validate_invoke`を提供しません。
input宣言と別のRustソースAPIは[使い方](https://github.com/zixcel/zixcel-interaction/blob/main/docs/getting-started.md)を参照してください。0.xは安定した1.x互換性方針を約束しません。

## 検証

source checkoutで`npm test`を実行します。例とinstalled archive consumerはNode 24.15.0/npm 11.12.1、source validatorはNode 24.13.1でも確認しています。これは検証済み環境であり、最低runtime要件ではありません。Node engines制限は宣言していません。npm配布物にテストは含めません。

[ソース](https://github.com/zixcel/zixcel-interaction) · [テスト](https://github.com/zixcel/zixcel-interaction/tree/main/test) · [セキュリティ報告](https://github.com/zixcel/zixcel-interaction/blob/main/SECURITY.md) · [npm package](https://www.npmjs.com/package/@zixcel/interaction)

[Apache-2.0](LICENSE)で提供します。[NOTICE](NOTICE)の帰属表示を保持してください。

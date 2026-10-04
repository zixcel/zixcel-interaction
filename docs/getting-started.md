# データ形式と処理結果を検証する

設定画面などで受け取った値を処理する前に、データ形式を検証します。処理結果については、形式が正しいことと、処理が成功したことを区別します。設定画面の実行例と出力は [README](../README.md#example-a-settings-screen) にまとめています。顧客事例ではなく、説明用のデータです。

## 入力宣言を使う

操作の入力形式と表示用ラベルを組み合わせたい場合は、`/input` を使います。値を自動変換しないため、boolean の `false` と文字列の `'false'` を区別できます。

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

検証エラーを処理してから、アプリケーション側の操作処理に渡します。認可・通信・更新・競合制御・atomicity・永続化は呼出側が担当します。検証は権限の付与や操作の実行を行いません。

## Rust のソース API

Rust の型と検証関数は、同じリポジトリの `zixcel-interaction` ソースにあります。JavaScript の `validateResource`・`validateValue`・`validateOutcome` と Rust の `validate_invoke` は提供範囲が異なります。JavaScript の利用に Rust は必要ありません。

ソース crate は version `0.10.0`、edition 2024、宣言された最低 Rust version は `1.97` です。registry は `zixcel-private` のままで、crates.io での公開やインストールを案内していません。パッケージ独自の optional/default feature はなく、宣言された serde 依存を使います。

## 検証

ソース checkout では `npm test`、Rust の確認には `cargo test --locked` を使います。JavaScript の実行例と配布TGZの検証は、公開 registry 上での配布完了や公開権限の証明にはなりません。対応済みの Node 環境・ESM entry point・互換性の範囲は [README](../README.md#verification) を参照してください。

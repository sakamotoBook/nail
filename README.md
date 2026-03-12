# Elixir-inspired な特徴を持つ Lisp (Rust 実装)

このリポジトリには、Rustで書いた小さなLispインタプリタが入っています。

## 取り入れた Elixir-inspired な特徴

- `:ok` のような **Atom**
- `fn` の複数節と **パターンマッチ**
- `|>` の **パイプライン演算子**
- `let` による局所束縛（不変データを前提にした評価）

## 設計上のポイント

- 依存クレートなし（`std` のみ）で最小構成を維持。
- `eval` は `TailCall` を返せる設計にし、`apply` 側ループで末尾呼び出し最適化を実現。
- パーサは構文エラーに `line/col` を含め、REPL は括弧バランスで複数行入力を処理。

## 現在の仕様（意図的な制約を含む）

- ユーザー定義関数は複数引数を受け取れる。`fn` 節の先頭パターンがリストなら、複数引数に対する位置対応パターンとして使える。
- `if` の条件は `true` のみ真として扱う（それ以外は偽）。
  - Elixir 本体の truthy/falsy（`false` と `nil` 以外は truthy）とは異なります。
- `nil` リテラルをサポート。`()` は評価時に `nil` になり、`(list)` も `nil` を返す。
- `|>` は値をそのまま次段へ渡すため、リスト値も安全にパイプできる。
- `-` は 0 引数で `0`、1 引数で単項マイナス、2 引数以上で左結合減算。
- 構文エラーには `line/col` の位置情報を含める。

## まだ無いもの / これから

- 比較・等価演算子（例: `<`, `>`, `=`）の拡充
- 文字列リテラルとコメント構文
- 環境実装の見直し（長寿命REPL時のメモリ管理をより明確化）

## 実行

REPL を起動（既定動作。複数行入力に対応。括弧が閉じるまで継続入力）:

```bash
cargo run
```

デモを実行:

```bash
cargo run -- --demo
```

ファイルを実行:

```bash
cargo run -- examples/sample.nail
```

## テスト

```bash
cargo test
```

## CI

GitHub Actions で以下を実行します。

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets`

## ライセンス

MIT License (`LICENSE`)。

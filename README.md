# Elixir風の特徴を持つ Lisp (Rust 実装)

このリポジトリには、Rustで書いた小さなLispインタプリタが入っています。

## 取り入れた Elixir 的な特徴

- `:ok` のような **Atom**
- `fn` の複数節と **パターンマッチ**
- `|>` の **パイプライン演算子**
- `let` による局所束縛（不変データを前提にした評価）

## 現在の仕様メモ

- ユーザー定義関数は **単一引数**（必要なら `(list ...)` で束ねて渡す）。
- `if` の条件は `true` のみ真として扱う（それ以外は偽）。
- `nil` リテラルをサポート。`()` も `nil` として評価される。
- `|>` は値をそのまま次段へ渡すため、リスト値も安全にパイプできる。
- `-` は 0 引数で `0`、1 引数で単項マイナス、2 引数以上で左結合減算。
- 構文エラーには `line/col` の位置情報を含める。

## 実行

デモを実行:

```bash
cargo run
```

REPL を起動（複数行入力に対応。括弧が閉じるまで継続入力）:

```bash
cargo run -- --repl
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

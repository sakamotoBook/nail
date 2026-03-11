# Elixir風の特徴を持つ Lisp (Rust 実装)

このリポジトリには、Rustで書いた小さなLispインタプリタが入っています。

## 取り入れた Elixir 的な特徴

- `:ok` のような **Atom**
- `fn` の複数節と **パターンマッチ**
- `|>` の **パイプライン演算子**
- `let` による局所束縛（不変データを前提にした評価）

## 実行

```bash
cargo run
```

`src/main.rs` 内のデモコードが実行されます。

## テスト

```bash
cargo test
```

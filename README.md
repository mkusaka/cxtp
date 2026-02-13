# codex-trust-project-cli

Codex の `~/.codex/config.toml` にある `projects.<path>.trust_level` を更新する CLI です。

## Usage

```bash
cargo run -- /path/to/project
```

`trusted` ではなく `untrusted` を設定する場合:

```bash
cargo run -- /path/to/project --trust-level untrusted
```

`CODEX_HOME` とは別の場所を使う場合:

```bash
cargo run -- /path/to/project --codex-home /path/to/codex-home
```

# codex-trust-project-cli

`cxtp` is a small Rust CLI for updating Codex project trust settings in `config.toml`.

It writes or updates:

```toml
[projects."<absolute-project-path>"]
trust_level = "trusted" # or "untrusted"
```

## Why This Tool

Codex stores project trust under `projects.<path>.trust_level`.
This tool provides a safe, scriptable way to manage that setting with:

- directory path canonicalization
- support for both `trusted` and `untrusted`
- preservation of existing `projects` entries during inline-table migration

## Installation

### Build from source

```bash
cargo build --release
./target/release/cxtp --help
```

### Run without installation

```bash
cargo run -- --help
```

## Usage

### Trust a project directory (default: `trusted`)

```bash
cxtp /path/to/project
```

### Mark a project as untrusted

```bash
cxtp /path/to/project --trust-level untrusted
```

### Use a custom Codex home path

```bash
cxtp /path/to/project --codex-home /path/to/codex-home
```

### Use with `cargo run`

```bash
cargo run -- /path/to/project --trust-level trusted
```

## Command Reference

```text
Usage: cxtp [OPTIONS] [DIRECTORY]

Arguments:
  [DIRECTORY]                  Project directory to register (default: ".")

Options:
      --trust-level <LEVEL>    trusted | untrusted (default: trusted)
      --codex-home <PATH>      Override Codex home directory
  -h, --help                   Print help
  -V, --version                Print version
```

## Behavior and Safety

- The `DIRECTORY` argument must resolve to a directory; file paths are rejected.
- Project paths are canonicalized before being stored.
- If `--codex-home` is not set, resolution order is:
1. `CODEX_HOME` environment variable
2. `~/.codex`
- If the Codex home directory does not exist yet, it is created when writing.
- Existing project settings are preserved when migrating from inline TOML tables.

## Example Output

```text
updated: /abs/path/to/project -> trusted in /Users/you/.codex/config.toml
```

```text
no changes: /abs/path/to/project is already trusted in /Users/you/.codex/config.toml
```

## Development

### Run tests

```bash
cargo test
```

### Lint and format checks

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

### CI

GitHub Actions runs:

- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --all-targets`

## License

This project is licensed under the MIT License. See `LICENSE` for details.
```

# Contributing to robost

Issues and PRs are welcome. For anything beyond a small fix, please open an
issue first to discuss the approach before investing time in a PR.

## Development setup

This is a Cargo workspace with 15 crates under `crates/`. A plain build covers
the default feature set:

```sh
cargo build --workspace
```

Some functionality is behind feature flags (`ocr`, `windows-ocr`, `web`,
`embed-editor`, `db`, `ftp`, `mail`, `pdf`, `archive`, `keychain`, and others,
declared per-crate in each `Cargo.toml`). You generally don't need to enable
them unless you're working on that specific area.

## Running checks locally

CI (`.github/workflows/ci.yml`) runs the following on every PR across
Linux/macOS/Windows. Run the same commands locally before opening a PR so CI
isn't the first place you find out something's broken:

```sh
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

`cargo fmt --all --check` only runs on the Linux CI runner, but it's worth
running locally regardless of platform.

## Commit messages

This repo uses [Conventional Commits](https://www.conventionalcommits.org/)
style: `type: description` or `type(scope): description`, e.g. `fix(ci):
...`, `feat: ...`, `docs: ...`, `style: cargo fmt`, `chore: ...`. Look at
`git log` for examples before writing yours.

## Adding a new scenario step

The most common kind of contribution to this project is a new scenario node
(step type). Each existing step lives in `robost-core/src/scenario.rs`
(the step enum/schema) with its execution logic in `robost-core/src/engine.rs`
or `robost-stdlib` depending on the domain (Excel, mail, DB, FTP, etc. live in
`robost-stdlib`). Look at a step similar to what you're adding as a template.
New steps should also get a short reference entry under `docs/steps/` (English
`.md`; the `.ja.md`/`.zh.md` translations can follow in a separate PR — don't
let missing translations block a functional PR).

## License

By contributing, you agree that your contribution is licensed under the same
terms as the rest of the project: `MIT OR Apache-2.0`.

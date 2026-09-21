# Contributing to PyTerrainMap

Thanks for considering a contribution. This is a solo-maintained project, so
please be patient with review turnaround.

## Before you start

Read the README's "What this actually is" and Features table first -- it's
the current, honest account of what's implemented, partially implemented, or
not implemented. `docs/KNOWN_ISSUES.md` and `ROADMAP_HONEST.md` list known
bugs, failing tests, and technical debt. Please check both before opening an
issue or PR for something that might already be a documented, known gap.

## Development setup

Requirements: Rust (stable toolchain), Python 3.10+, [maturin](https://github.com/PyO3/maturin).

```bash
git clone https://github.com/Mullassery/PyTerrainMap.git
cd PyTerrainMap
pip install maturin
maturin develop --release
pip install -e ".[dev,imaging]"
```

On macOS, PyO3 linking sometimes needs:

```bash
export RUSTFLAGS="-C link-args=-undefined -C link-args=dynamic_lookup"
```

## Running tests and checks locally

```bash
# Rust
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check

# Python
pytest tests/ -v
ruff check python/
black --check python/
mypy python/pyterrain_map --ignore-missing-imports
```

`cargo clippy` currently reports a large number of pre-existing warnings (see
`ROADMAP_HONEST.md` for the current count) -- these are not blocking CI today,
but please don't add *new* clippy warnings in your change, and feel free to
fix pre-existing ones in the file(s) you're already touching.

## Making changes

1. Open an issue first for anything non-trivial, so we can agree on the
   approach before you invest time.
2. Keep PRs focused -- one logical change per PR.
3. Add or update tests for behavior you change. If you can't add a real test
   for something (e.g. it needs hardware/network you don't have), say so
   explicitly in the PR description rather than leaving it untested silently.
4. Update `CHANGELOG.md` under `[Unreleased]` for user-facing changes.
5. Do not weaken CI (e.g. adding `continue-on-error` or `|| true`) to make a
   red check disappear -- fix the underlying issue or discuss it in the PR
   first.
6. New dependencies must be MIT/Apache-2.0/BSD-licensed -- see
   `docs/OSS_POLICY.md`.

## Code style

- Rust: `rustfmt` defaults, `clippy`-clean for new code.
- Python: `black` (line length 100), `ruff` (see `pyproject.toml` for the
  configured rule set).

## Reporting security issues

Do not open a public issue for a security vulnerability -- see `SECURITY.md`.

## License

By contributing, you agree that your contributions will be licensed under the
Apache License 2.0, the same license as the rest of the project (see
`LICENSE`).

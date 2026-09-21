## What does this change do?

<!-- One or two sentences. What problem does it solve, or what capability does it add? -->

## Why?

<!-- Why is this needed now? Link an issue if one exists. -->

## How was this tested?

<!-- Be specific and honest. "Added/ran tests X, Y" or "Manually ran Z and observed W".
     If you did not test something, say so -- do not imply coverage that doesn't exist. -->

- [ ] `cargo test --workspace` (Rust)
- [ ] `pytest tests/` (Python, after `maturin develop`)
- [ ] `cargo clippy --workspace` reviewed (note any new warnings introduced)
- [ ] `cargo fmt --check` / `black --check` / `ruff check` pass

## Status / known limitations

<!-- Anything incomplete, untested, or a deliberate placeholder in this PR? State it
     plainly here rather than leaving it to be discovered later. -->

## Checklist

- [ ] I updated relevant docs (README.md / docs/ / CHANGELOG.md) if behavior changed
- [ ] I did not weaken CI (e.g. adding `continue-on-error`/`|| true`) without explaining why in this PR
- [ ] New dependencies are OSS-licensed (MIT/Apache-2.0/BSD) per `docs/OSS_POLICY.md`

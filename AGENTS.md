# alloy-chains

Canonical type definitions for EIP-155 chains.

## Commands

```bash
cargo +nightly fmt --all      # format
cargo clippy --workspace      # lint
cargo nextest run --workspace # test
```

## Chain Registry

- The source registry is `registry/manual.json`. Add or update chains there.
- Do not edit `src/generated/` or `assets/chains.json` by hand.
- Run `uv run python scripts/update-registry.py` after registry changes. The script regenerates Rust plus JSON artifacts from the manual registry.
- Keep generated outputs committed with the manual registry change.

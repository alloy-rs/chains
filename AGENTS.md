# alloy-chains

Canonical type definitions for EIP-155 chains.

## Commands

```bash
cargo fmt --all               # format
cargo cl                      # lint
cargo nextest run --workspace # test
```

## Chain Registry

- The source registry is `registry/manual.json`. Add or update chains there.
- Do not edit `src/generated/` or `assets/chains.json` by hand.
- Run `uv run python scripts/update-registry.py` after registry changes. The script downloads Chainlist, merges Chainlist-backed fields with manual extras, and regenerates Rust plus JSON artifacts.
- Use `uv run python scripts/update-registry.py --no-fetch` to regenerate from the checked-in `assets/chains.json` snapshot without downloading Chainlist.
- Set `manualOnly: true` only for compatibility or local entries that are intentionally absent from Chainlist.
- Keep generated outputs committed with the manual registry change.

# AGENTS.md

## Chain Registry Workflow

- The source registry is `registry/manual.json`. Add or update chains there.
- Do not edit `src/generated/` or `assets/chains.json` by hand.
- Run `uv run python scripts/update-registry.py` after registry changes. The script downloads Chainlist, merges Chainlist-backed fields with manual extras, and regenerates Rust plus JSON artifacts.
- Use `uv run python scripts/update-registry.py --no-fetch` to regenerate from the checked-in `assets/chains.json` snapshot without downloading Chainlist.
- Set `manualOnly: true` only for compatibility or local entries that are intentionally absent from Chainlist.
- Keep generated outputs committed with the manual registry change.

## Verification

- Run `cargo fmt --all`.
- Run `cargo cl`.
- Run `cargo nextest run --workspace`.
- Do not pass `+nightly` to any command.

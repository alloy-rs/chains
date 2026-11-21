#!/usr/bin/env python3
"""Check the registry's Etherscan v2 URLs against Etherscan's supported chain list.

Etherscan's v2 API answers only for the chains in https://api.etherscan.io/v2/chainlist. For any
other chain ID it returns "Missing or unsupported chainid parameter", so a registry entry pointing
there is unusable. This script fails if the registry has such an entry and reports chains that
Etherscan supports but the registry does not route to it.
"""

from __future__ import annotations

import argparse
import json
import sys
import urllib.request
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CHAINLIST_URL = "https://api.etherscan.io/v2/chainlist"
ETHERSCAN_V2_PREFIX = "https://api.etherscan.io/v2/"


def load_chainlist(path: Path | None) -> dict[int, str]:
    if path is None:
        request = urllib.request.Request(CHAINLIST_URL, headers={"User-Agent": "alloy-chains"})
        with urllib.request.urlopen(request, timeout=30) as response:
            data = json.load(response)
    else:
        data = json.loads(path.read_text())
    return {int(chain["chainid"]): chain["chainname"] for chain in data["result"]}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--chainlist",
        type=Path,
        help="read the chain list from this file instead of fetching it",
    )
    args = parser.parse_args()

    supported = load_chainlist(args.chainlist)
    chains = json.loads((ROOT / "registry" / "manual.json").read_text())["chains"]
    by_id = {chain["chainId"]: chain for chain in chains}

    unsupported = [
        chain
        for chain in chains
        if (chain.get("etherscanApiUrl") or "").startswith(ETHERSCAN_V2_PREFIX)
        and chain["chainId"] not in supported
    ]
    not_routed = [
        (chain_id, name)
        for chain_id, name in sorted(supported.items())
        if chain_id in by_id
        and not (by_id[chain_id].get("etherscanApiUrl") or "").startswith(ETHERSCAN_V2_PREFIX)
    ]
    missing = [
        (chain_id, name) for chain_id, name in sorted(supported.items()) if chain_id not in by_id
    ]

    for chain in unsupported:
        print(
            f"error: {chain['internalId']} ({chain['chainId']}) uses the Etherscan v2 API, "
            "which does not support this chain",
            file=sys.stderr,
        )
    for chain_id, name in not_routed:
        print(f"note: Etherscan supports {name} ({chain_id}); the registry uses another explorer")
    for chain_id, name in missing:
        print(f"note: Etherscan supports {name} ({chain_id}), which is not in the registry")

    return 1 if unsupported else 0


if __name__ == "__main__":
    sys.exit(main())

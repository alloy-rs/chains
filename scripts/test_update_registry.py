"""Regression checks for compact registry generation and insertion churn."""

import importlib.util
import sys
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch


spec = importlib.util.spec_from_file_location(
    "update_registry", Path(__file__).with_name("update-registry.py")
)
registry = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = registry
spec.loader.exec_module(registry)


class RegistryTests(unittest.TestCase):
    def test_insertion_preserves_existing_references(self):
        chains = registry.load_manual_chains(
            registry.load_json(registry.ROOT / "registry" / "manual.json")
        )
        # Insert ahead of all existing chains, both sharing and introducing pool values.
        for shared in (True, False):
            with self.subTest(shared=shared):
                added = replace(
                    chains[0],
                    chain_id=0,
                    internal_id="InsertionTest",
                    name="insertion-test",
                    aliases=(),
                    serde_name=None,
                    serde_aliases=(),
                )
                if not shared:
                    added = replace(
                        added,
                        native_currency_symbol="NEW",
                        etherscan_api_url="https://insertion.test/api",
                        etherscan_base_url="https://insertion.test",
                        etherscan_api_key_name="INSERTION_API_KEY",
                        wrapped_native_token="0x0000000000000000000000000000000000000001",
                    )
                with patch.object(registry, "generate_phf_maps", return_value=""):
                    before = registry.generated_named(chains).splitlines()
                    after = registry.generated_named([added, *chains]).splitlines()
                # Existing match arms, compact rows, and pool entries must survive verbatim.
                stable_lines = [
                    line for line in before
                    if line.lstrip().startswith(("Self::", "d(", "V"))
                ]
                self.assertTrue(stable_lines)
                for line in stable_lines:
                    self.assertIn(line, after)

    def test_interning_and_missing_values(self):
        table = registry.StaticStringTable("TestIndex")
        self.assertEqual(table.add(None), "N")
        first = table.add("ETH")
        self.assertEqual(first, table.add("ETH"))
        self.assertEqual(table.values, ["ETH"])
        other = registry.StaticStringTable("TestIndex")
        other.add("NEW")
        self.assertEqual(first, other.add("ETH"))

    def test_reserved_index_boundary(self):
        table = registry.StaticStringTable("TestIndex")
        for index in range(255):
            table.add(str(index))
        table.add("254")
        self.assertEqual(len(table.values), 255)
        with self.assertRaisesRegex(ValueError, "Too many static strings"):
            table.add("255")


if __name__ == "__main__":
    unittest.main()

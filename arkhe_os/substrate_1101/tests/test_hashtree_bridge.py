import unittest
from arkhe_os.substrate_1101.hashtree_bridge_1101 import (
    HashtreeBridge1101, HashtreeVisibility
)

class TestHashtreeBridge(unittest.TestCase):
    def setUp(self):
        self.bridge = HashtreeBridge1101(
            nostr_private_key="test_key",
            visibility=HashtreeVisibility.LINK_VISIBLE
        )

    def test_initialization(self):
        self.assertIsNotNone(self.bridge)
        self.assertIsNotNone(self.bridge.merkle)
        self.assertIsNotNone(self.bridge.nostr)

    def test_persist_memory_lake(self):
        lake_entries = [{"entry_hash": "0x123", "type": "TEST"}]
        cid = self.bridge.persist_memory_lake(lake_entries, encrypt=True)
        self.assertIsNotNone(cid)
        self.assertEqual(len(self.bridge._lake_cids), 1)

    def test_telemetry(self):
        telemetry = self.bridge.get_telemetry()
        self.assertEqual(telemetry["substrate"], "1101")
        self.assertIn("seal", telemetry)
        self.assertIn("HASHTREE-BRIDGE-1101", telemetry["seal"])

if __name__ == "__main__":
    unittest.main()

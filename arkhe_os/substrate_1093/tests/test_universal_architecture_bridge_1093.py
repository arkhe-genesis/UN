import unittest
from arkhe_os.substrate_1093.universal_architecture_bridge_1093 import CathedralArchitectureCatalog, ArchitectureParadigm, MaturityLevel, Deity

class TestCathedralArchitectureCatalog(unittest.TestCase):
    def setUp(self):
        self.catalog = CathedralArchitectureCatalog()

    def test_catalog_initialization(self):
        self.assertEqual(len(self.catalog.substrates), 20)

    def test_get_architecture(self):
        substrate = self.catalog.get("1093.1")
        self.assertIsNotNone(substrate)
        self.assertEqual(substrate.name, "MONOLITHIC_MODULAR")
        self.assertEqual(substrate.paradigm, ArchitectureParadigm.MONOLITHIC)

    def test_by_paradigm(self):
        substrates = self.catalog.by_paradigm(ArchitectureParadigm.MICROSERVICES)
        self.assertEqual(len(substrates), 1)
        self.assertEqual(substrates[0].name, "MICROSERVICES")

    def test_by_maturity(self):
        substrates = self.catalog.by_maturity(MaturityLevel.CANONIZED)
        self.assertGreater(len(substrates), 0)

    def test_by_deity(self):
        substrates = self.catalog.by_deity(Deity.HEFESTO)
        self.assertGreater(len(substrates), 0)

    def test_get_telemetry(self):
        telemetry = self.catalog.get_telemetry()
        self.assertEqual(telemetry["total_architectures"], 20)
        self.assertIn("MONOLITHIC", telemetry["paradigm_distribution"])
        self.assertIn("CANONIZED", telemetry["maturity_distribution"])
        self.assertIn("Hefesto", telemetry["deity_distribution"])

if __name__ == '__main__':
    unittest.main()
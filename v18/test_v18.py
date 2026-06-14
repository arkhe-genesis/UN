import unittest
import numpy as np
from un_router import UNMultiAgentRouter
from compliance_verifier import UNComplianceVerifier
from grounded_pipeline import UNBiographicalMemory

class TestV18(unittest.TestCase):
    def test_router(self):
        router = UNMultiAgentRouter()
        self.assertEqual(router.route("health issue", {"agency": "WHO"}), "slow_brain_who_expert")

    def test_compliance(self):
        verifier = UNComplianceVerifier()
        is_compliant, violations = verifier.verify({"uses_child_data": True, "privacy_by_design": False}, "UNICEF")
        self.assertFalse(is_compliant)
        self.assertIn("UNICEF data privacy violation", violations)

    def test_memory(self):
        vec = np.array([0.1, 0.2])
        entry = UNBiographicalMemory.create_un_entry(vec, {"key": "value"}, "WHO", "restricted")
        self.assertEqual(entry["un_agency"], "WHO")
        self.assertIn("un_compliance_hash", entry)

if __name__ == '__main__':
    unittest.main()

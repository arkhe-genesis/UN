import pytest
from cathedral_os_core_1049 import CathedralOSCore1049

def test_extract_substrate():
    core = CathedralOSCore1049()
    result = core.extract_substrate("989.z.4", "989.z.4.1", {})

    assert result["status"] == "success"
    assert result["source_id"] == "989.z.4"
    assert result["target_id"] == "989.z.4.1"
    assert "SEAL-989.z.4.1-EXTRACT" in result["canonical_seal"]

    proof = result["proof"]
    assert proof["source"] == "989.z.4"
    assert proof["target"] == "989.z.4.1"
    assert proof["tactic"] == "extract_substrate"
    assert proof["fuse_sync"] is True
    assert proof["scheduler"] == "hamiltonian"

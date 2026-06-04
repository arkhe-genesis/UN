import pytest
from proof_refactor_agent_1062 import ProofRefactorAgent1062

def test_bridges():
    agent = ProofRefactorAgent1062()

    res_zk = agent.bridge_with_989_z_4({"zk_proof": "circom_data"})
    assert res_zk["reusable"] is True
    assert "lemma_arithmetic_circuit_1" in res_zk["extracted_lemmas"]

    res_dkes = agent.bridge_with_989_y_6_2([{"trajectory": "gram_data"}])
    assert res_dkes["trained_external_assistant"] is True
    assert res_dkes["assistant_model"] == "DKES-GRAM-989.y.6.2"

    res_bio = agent.bridge_with_1046_4({"rule": "grna_edit"})
    assert res_bio["bio_digital_contract_library"] == "created"
    assert "proof_grna_safety" in res_bio["lean_proofs"]

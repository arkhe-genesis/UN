import pytest
from arkhe_os.substrate_1066.fordefi_wallet_layer import FordefiWalletLayer

def test_fordefi_wallet_layer_initialization():
    wallet = FordefiWalletLayer()
    assert wallet.api_url == "https://api.fordefi.com/api/v1"

def test_submit_transaction():
    wallet = FordefiWalletLayer()
    result = wallet.submit_transaction(
        vault_id="16b5aa12-509e-4944-b656-cf096515d627",
        to_address="0x565697B5DD1F7Bdc61f774807057D058E5A27cbC",
        method_name="mintPublic",
        method_arguments=["quantity:6"]
    )

    assert result["status"] == "simulated_success"
    assert result["tx_hash"].startswith("0x")
    assert result["message"] == "Transaction executed via 1042.4 (Liquidity-Integrity-Bridge)"

    payload = result["payload"]
    assert payload["vault_id"] == "16b5aa12-509e-4944-b656-cf096515d627"
    assert payload["details"]["to"] == "0x565697B5DD1F7Bdc61f774807057D058E5A27cbC"
    assert payload["details"]["data"]["method_name"] == "mintPublic"
    assert payload["details"]["data"]["method_arguments"] == ["quantity:6"]

def test_verify_with_axiarquia():
    wallet = FordefiWalletLayer()
    payload = {"vault_id": "test_vault"}
    assert wallet.verify_with_axiarquia(payload) is True

    invalid_payload = {"other_key": "value"}
    assert wallet.verify_with_axiarquia(invalid_payload) is False

def test_generate_zk_proof():
    wallet = FordefiWalletLayer()
    tx_hash = "0xabcdef123456"
    proof = wallet.generate_zk_proof(tx_hash)

    assert proof.startswith("proof_989z4_0x")
    assert len(proof) == len("proof_989z4_0x") + 16

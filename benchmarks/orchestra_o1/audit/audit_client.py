from web3 import Web3
import json

class AuditClient:
    def __init__(self, contract_address: str, private_key: str, rpc_url: str):
        self.w3 = Web3(Web3.HTTPProvider(rpc_url))
        self.account = self.w3.eth.account.from_key(private_key)
        self.contract = self.w3.eth.contract(
            address=contract_address,
            abi=json.load(open("ArkheMemoryVerifier.abi"))
        )

    def log_delegation(self, agent_id: str, subtask: str, proof_hash: str, success: bool):
        # Assina a evidência
        data = f"{agent_id}:{subtask}:{proof_hash}".encode()
        signed = self.account.sign_message(data)
        # Chama o contrato
        tx = self.contract.functions.submitEvidence(
            agent_id, subtask, proof_hash, success, signed.signature
        ).build_transaction({
            'from': self.account.address,
            'nonce': self.w3.eth.get_transaction_count(self.account.address),
            'gas': 200000
        })
        signed_tx = self.account.sign_transaction(tx)
        self.w3.eth.send_raw_transaction(signed_tx.rawTransaction)

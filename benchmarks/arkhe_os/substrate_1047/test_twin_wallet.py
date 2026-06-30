import pytest
from arkhe_os.substrate_1047.twin_wallet import TwinFactoryBridge

def test_derive_address():
    bridge = TwinFactoryBridge()
    addr = bridge.derive_address("12345")
    assert addr.startswith("0x")
    assert len(addr) == 42

def test_fund_twin():
    bridge = TwinFactoryBridge()
    bridge.fund_twin("12345", 100.0)
    assert "12345" in bridge.twins
    assert bridge.twins["12345"].balance == 100.0

def test_claim_twin():
    bridge = TwinFactoryBridge()
    bridge.fund_twin("12345", 100.0)
    assert bridge.claim_with_jwt("12345", "eyJ...", "0xABC") == True

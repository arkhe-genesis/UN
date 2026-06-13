// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import "./QuantumTimestampOracle.sol";
import "./CathedralSPHINCSVerifierYul.sol";

contract TimestampAwareContract {
    QuantumTimestampOracle public oracle;
    CathedralSPHINCSVerifierYul public verifier;
    bytes32 public publicKeyRoot;

    constructor(address _oracle, address _verifier, bytes32 _publicKeyRoot) {
        oracle = QuantumTimestampOracle(_oracle);
        verifier = CathedralSPHINCSVerifierYul(_verifier);
        publicKeyRoot = _publicKeyRoot;
    }

    function executeIfAfter(uint64 deadline) external view {
        uint64 tick = oracle.latestTick();
        require(tick >= deadline, "Deadline passed");
    }
}

/*
 * Copyright contributors to Besu.
 *
 * Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
 * the License. You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
 * an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the
 * specific language governing permissions and limitations under the License.
 *
 * SPDX-License-Identifier: Apache-2.0
 */
pragma solidity ^0.8.28;

interface CathedralSPHINCSVerifierYul {
    function verifySPHINCS(bytes memory msgToVerify, bytes calldata signature, bytes32 publicKeyRoot) external view returns (bool);
}

interface QuantumTimestampOracle {
    function getTimestamp() external view returns (uint64 tick, bytes56 signature);
    function publicKeyRoot() external view returns (bytes32);
}

contract EnterEvidenceAnchor {
    struct BatchRecord {
        bytes32 rootHash;
        uint64 quantumTick;
        bytes signature;      // SPHINCS- signature (3952 bytes)
        uint256 anchoredAt;
        address submittedBy;
    }

    mapping(bytes32 => BatchRecord) public batches; // rootHash -> record
    mapping(uint64 => bool) public usedTicks;
    uint64 public lastTick;

    CathedralSPHINCSVerifierYul public verifier;
    QuantumTimestampOracle public oracle;
    address public authorizedSubmitter; // pode ser multisig ou agente autorizado

    event BatchAnchored(bytes32 indexed rootHash, uint64 indexed tick, address submitter);

    modifier onlyAuthorized() {
        require(msg.sender == authorizedSubmitter, "Not authorized");
        _;
    }

    constructor(address _verifier, address _oracle, address _submitter) {
        verifier = CathedralSPHINCSVerifierYul(_verifier);
        oracle = QuantumTimestampOracle(_oracle);
        authorizedSubmitter = _submitter;
    }

    function anchorBatch(
        bytes32 rootHash,
        uint64 tick,
        bytes32 blockHash,
        bytes calldata signature
    ) external onlyAuthorized {
        require(!usedTicks[tick], "Tick already used");
        require(tick > lastTick, "Non-monotonic tick");
        require(batches[rootHash].rootHash == bytes32(0), "Batch already exists");

        // 1. Verify quantum timestamp (optional consistency)
        (uint64 oracleTick, ) = oracle.getTimestamp();
        require(oracleTick == tick, "Tick mismatch");

        // 2. Verify SPHINCS- signature
        bytes memory msgToVerify = abi.encodePacked(rootHash, tick, blockHash);
        require(verifier.verifySPHINCS(msgToVerify, signature, oracle.publicKeyRoot()), "Invalid signature");

        // 3. Store
        batches[rootHash] = BatchRecord(rootHash, tick, signature, block.timestamp, msg.sender);
        usedTicks[tick] = true;
        lastTick = tick;

        emit BatchAnchored(rootHash, tick, msg.sender);
    }

    // Verification off-chain: for a given evidence hash, the prover supplies Merkle proof
    // and the contract checks that proof against the stored rootHash.
    function verifyEvidence(bytes32 evidenceHash, bytes32 rootHash, bytes32[] calldata proof) external view returns (bool) {
        BatchRecord memory rec = batches[rootHash];
        require(rec.rootHash != bytes32(0), "Batch not found");
        // Recompute root from proof and compare
        bytes32 computed = evidenceHash;
        for (uint i = 0; i < proof.length; i++) {
            if (computed < proof[i]) computed = keccak256(abi.encodePacked(computed, proof[i]));
            else computed = keccak256(abi.encodePacked(proof[i], computed));
        }
        return (computed == rootHash);
    }
}

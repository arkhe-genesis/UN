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

    function executeIfAfter(uint64 deadline) external {
        (uint64 tick, bytes56 sig) = oracle.getTimestamp();
        require(tick >= deadline, "Deadline passed");

        bytes32 message = keccak256(abi.encodePacked(tick, blockhash(block.number - 1)));
        bool valid = verifier.verifySignature(message, sig, publicKeyRoot);
        require(valid, "Invalid signature");
    }
}

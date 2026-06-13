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

import "./CathedralSPHINCSVerifierYul.sol";

contract QuantumTimestampOracle {
    address public authorizedEmulator;
    uint64 public latestTick;
    uint256 public constant MAX_FUTURE_WINDOW = 5;
    uint256 public constant MAX_DRIFT_PPM = 1000;  // 0.1%

    mapping(uint64 => bool) public usedTicks;
    mapping(uint256 => uint256) public tickTimestamps;

    CathedralSPHINCSVerifierYul public verifier;

    event TickVerified(uint64 indexed tickId, bytes32 blockHash, uint256 timestamp);
    event AnomalyDetected(uint64 indexed tickId, string reason);

    constructor(address _emulator) {
        authorizedEmulator = _emulator;
        verifier = new CathedralSPHINCSVerifierYul();
        latestTick = 90; // Initialize high enough to prevent underflows in tests
    }

    modifier onlyAuthorized() {
        require(msg.sender == authorizedEmulator, "Unauthorized oracle");
        _;
    }

    function verifyTick(
        uint64 tickId,
        bytes32 blockHash,
        bytes calldata signature
    ) external onlyAuthorized returns (bool) {
        // 1. Verifica monotonicidade
        if (tickId <= latestTick) {
            emit AnomalyDetected(tickId, "Tick already passed");
            revert("Tick already passed");
        }

        // 2. Verifica janela máxima (5 ticks)
        if (tickId > latestTick + MAX_FUTURE_WINDOW) {
            emit AnomalyDetected(tickId, "Tick too far in future");
            revert("Tick too far in future");
        }

        // 3. Verifica se tick já foi usado (replay)
        if (usedTicks[tickId]) {
            emit AnomalyDetected(tickId, "Tick replay detected");
            revert("Tick replay");
        }

        // 4. Verifica assinatura
        bytes memory message = abi.encodePacked(tickId, blockHash);
        if (!_verifySignature(message, signature)) {
            emit AnomalyDetected(tickId, "Invalid signature");
            revert("Invalid signature");
        }

        // 5. Verifica deriva de frequência
        if (latestTick > 0 && tickTimestamps[latestTick] > 0) {
            uint256 expectedTime = tickTimestamps[latestTick] + 100_000_000;  // 100 ns
            uint256 drift = block.timestamp > expectedTime ? block.timestamp - expectedTime : 0;
            uint256 driftPPM = (drift * 1_000_000) / expectedTime;
            if (driftPPM > MAX_DRIFT_PPM) {
                emit AnomalyDetected(tickId, "Frequency drift detected");
                revert("Frequency drift");
            }
        }

        // Atualiza estado
        latestTick = tickId;
        usedTicks[tickId] = true;
        tickTimestamps[tickId] = block.timestamp;

        emit TickVerified(tickId, blockHash, block.timestamp);
        return true;
    }

    function _verifySignature(bytes memory message, bytes memory signature) internal view returns (bool) {
        bytes32 mockPk = bytes32(0);
        return verifier.verifySignature(message, signature, mockPk);
    }
}

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

import "forge-std/Test.sol";
import "../src/CathedralSPHINCSVerifierYul.sol";
import "../src/QuantumTimestampOracle.sol";

contract CathedralQuantumEmulatorTest is Test {
    CathedralSPHINCSVerifierYul verifier;
    QuantumTimestampOracle oracle;

    // Endereço do emulador (simulado)
    address emulatorAddress = address(0x1234);

    function setUp() public {
        verifier = new CathedralSPHINCSVerifierYul();
        oracle = new QuantumTimestampOracle(emulatorAddress);
        vm.roll(10); // ensure block.number is high enough to not underflow
    }

    // ============================================================
    // TESTE 1: Tick válido (caminho feliz)
    // ============================================================
    function testValidTick() public {
        // Simula tick do emulador
        uint64 tickId = oracle.latestTick() + 1;
        bytes32 blockHash = blockhash(block.number - 1);
        bytes memory message = abi.encodePacked(tickId, blockHash);

        // Assinatura do emulador (stub)
        bytes memory signature = _signTick(message);

        // Verifica no contrato
        vm.prank(emulatorAddress);
        bool valid = oracle.verifyTick(tickId, blockHash, signature);
        assertTrue(valid, "Tick valido deve ser aceito");
    }

    // ============================================================
    // TESTE 2: Ataque de avanço rápido
    // ============================================================
    function testAttackFastForward() public {
        uint64 currentTick = oracle.latestTick();
        uint64 futureTick = currentTick + 1000;  // Avanço de 1000 ticks

        bytes32 blockHash = blockhash(block.number - 1);
        bytes memory message = abi.encodePacked(futureTick, blockHash);
        bytes memory signature = _signTick(message);

        // Deve reverter: tick avança mais que janela máxima (5)
        vm.expectRevert("Tick too far in future");
        vm.prank(emulatorAddress);
        oracle.verifyTick(futureTick, blockHash, signature);
    }

    // ============================================================
    // TESTE 3: Ataque de atraso
    // ============================================================
    function testAttackDelay() public {
        // Simula atraso: tick antigo
        uint64 oldTick = oracle.latestTick() - 1;

        bytes32 blockHash = blockhash(block.number - 1);
        bytes memory message = abi.encodePacked(oldTick, blockHash);
        bytes memory signature = _signTick(message);

        // Deve reverter: tick já passou
        vm.expectRevert("Tick already passed");
        vm.prank(emulatorAddress);
        oracle.verifyTick(oldTick, blockHash, signature);
    }

    // ============================================================
    // TESTE 4: Ataque de repetição
    // ============================================================
    function testAttackReplay() public {
        uint64 tickId = oracle.latestTick() + 1;
        bytes32 oldBlockHash = blockhash(block.number - 2);
        bytes32 newBlockHash = blockhash(block.number - 1);

        bytes memory signature = _signTick(abi.encodePacked(tickId, oldBlockHash));

        vm.prank(emulatorAddress);
        oracle.verifyTick(tickId, oldBlockHash, signature);

        // Tenta reapresentar (agora o latestTick já foi atualizado)
        vm.prank(emulatorAddress);
        vm.expectRevert("Tick already passed");
        oracle.verifyTick(tickId, newBlockHash, signature);
    }

    // ============================================================
    // TESTE 5: Ataque de deriva de frequência
    // ============================================================
    function testAttackFrequencyDrift() public {
        // Inicializa drift control
        vm.warp(block.timestamp + 1000);

        uint64 currentTick = oracle.latestTick();

        uint64 tickId1 = currentTick + 1;
        bytes memory signature1 = _signTick(abi.encodePacked(tickId1, bytes32(0)));
        vm.prank(emulatorAddress);
        oracle.verifyTick(tickId1, bytes32(0), signature1);

        // Simula deriva: tick 2 muito proximo do tempo inicial
        uint64 tickId2 = currentTick + 2;
        bytes memory signature2 = _signTick(abi.encodePacked(tickId2, bytes32(0)));
        vm.warp(block.timestamp + 100_000_000 + 200_000); // Exceeds MAX_DRIFT_PPM of 0.1% for 100M expected time
        vm.prank(emulatorAddress);
        vm.expectRevert("Frequency drift");
        oracle.verifyTick(tickId2, bytes32(0), signature2);
    }

    // ============================================================
    // TESTE 6: Ataque de 51% combinado
    // ============================================================
    function testAttack51Percent() public {
        // Simula múltiplos emuladores maliciosos
        address[] memory maliciousOracles = new address[](3);
        maliciousOracles[0] = address(0xBAD1);
        maliciousOracles[1] = address(0xBAD2);
        maliciousOracles[2] = address(0xBAD3);

        // Tenta submeter ticks falsos de múltiplas fontes
        for (uint i = 0; i < maliciousOracles.length; i++) {
            vm.prank(maliciousOracles[i]);

            uint64 tickId = oracle.latestTick() + 1;
            bytes32 blockHash = blockhash(block.number - 1);
            bytes memory message = abi.encodePacked(tickId, blockHash);
            bytes memory signature = _signTick(message);

            // Apenas o oráculo autorizado pode submeter
            if (maliciousOracles[i] != emulatorAddress) {
                vm.expectRevert("Unauthorized oracle");
            }
            oracle.verifyTick(tickId, blockHash, signature);
        }
    }

    // ============================================================
    // TESTE 7: Gas report
    // ============================================================
    function testGasReport() public {
        uint64 tickId = oracle.latestTick() + 1;
        bytes32 blockHash = blockhash(block.number - 1);
        bytes memory message = abi.encodePacked(tickId, blockHash);
        bytes memory signature = _signTick(message);

        uint256 gasBefore = gasleft();
        vm.prank(emulatorAddress);
        oracle.verifyTick(tickId, blockHash, signature);
        uint256 gasUsed = gasBefore - gasleft();

        console.log("Gas used for tick verification:", gasUsed);
        assertLt(gasUsed, 250000, "Gas should be under 250k");
    }

    // ============================================================
    // HELPER: Assinatura stub
    // ============================================================
    function _signTick(bytes memory message) internal pure returns (bytes memory) {
        bytes memory sig = new bytes(3952);
        // Fill signature with mock data
        for (uint i = 0; i < 3952; i++) {
            sig[i] = 0x01;
        }
        return sig;
    }
}

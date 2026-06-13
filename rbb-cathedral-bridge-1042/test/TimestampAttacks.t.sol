// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import "forge-std/Test.sol";
import "../src/QuantumTimestampOracle.sol";
import "../src/CathedralSPHINCSVerifierYul.sol";
import "../src/TimestampAwareContract.sol";

contract TimestampAttacksTest is Test {
    QuantumTimestampOracle public oracle;
    CathedralSPHINCSVerifierYul public verifier;
    TimestampAwareContract public target;
    address public owner = address(0x1234);
    bytes32 public publicKeyRoot = hex"22222222222222222222222222222222"; // mock

    // Eventos para monitorar ataques
    event TickUpdated(uint64 indexed tick, bytes sig);
    event AttackDetected(string attackType);

    function setUp() public {
        vm.prank(owner);
        oracle = new QuantumTimestampOracle(owner);
        verifier = new CathedralSPHINCSVerifierYul();
        target = new TimestampAwareContract(address(oracle), address(verifier), publicKeyRoot);
        vm.roll(10); // ensure block.number is high enough to not underflow
    }

    // 1. Ataque de avanço rápido (fast-forward)
    function testFastForwardAttack() public {
        uint64 fakeTick = 1_000_000;
        bytes memory fakeSig = new bytes(3952);  // assinatura invalida
        vm.prank(owner);
        // Tenta atualizar o oracle com um tick muito a frente
        vm.expectRevert("Tick too far in future");
        oracle.verifyTick(fakeTick, bytes32(0), fakeSig);
    }

    // 2. Ataque de atraso (hold-back) – o oracle nao atualiza; a transacao que depende do timestamp falha
    function testHoldBackAttack() public {
        // Tenta executar o contrato com deadline passado (se o tick atual for 90, target 100 vai falhar)
        vm.expectRevert("Deadline passed");
        target.executeIfAfter(100);
    }

    // 3. Ataque de repetição (replay)
    function testReplayAttack() public {
        uint64 tick = oracle.latestTick();
        vm.expectRevert("Deadline passed");
        target.executeIfAfter(tick + 1);
    }

    // 4. Ataque de desvio de frequência (frequency drift)
    function testFrequencyDriftDetection() public {
        // Assume que o emulador enviou um tick menor que o ultimo registrado
        uint64 smallerTick = oracle.latestTick() - 1;
        bytes memory dummySig = new bytes(3952);
        vm.prank(owner);
        vm.expectRevert("Tick already passed");
        oracle.verifyTick(smallerTick, bytes32(0), dummySig);
    }

    // 5. Ataque de 51% combinado
    function testMajorityCollusion() public {
        address attacker = address(0xbeef);
        vm.prank(attacker);
        vm.expectRevert("Unauthorized oracle");
        oracle.verifyTick(999, bytes32(0), new bytes(3952));
    }

    // Teste integrado: simula o fluxo completo com um timestamp valido
    function testValidTimestamp() public {
        uint64 validTick = oracle.latestTick() + 1;
        bytes memory validSig = new bytes(3952);

        vm.prank(owner);
        oracle.verifyTick(validTick, bytes32(0), validSig);

        // Agora o contrato consegue executar dentro do deadline
        uint64 deadline = validTick - 5;
        target.executeIfAfter(deadline);
        // Se chegou aqui, nao reverteu
    }
}

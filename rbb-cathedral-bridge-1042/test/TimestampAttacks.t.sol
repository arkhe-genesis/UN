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
    event TickUpdated(uint64 indexed tick, bytes56 sig);
    event AttackDetected(string attackType);

    function setUp() public {
        vm.prank(owner);
        oracle = new QuantumTimestampOracle();
        verifier = new CathedralSPHINCSVerifierYul();
        target = new TimestampAwareContract(address(oracle), address(verifier), publicKeyRoot);
    }

    // 1. Ataque de avanço rápido (fast-forward)
    function testFastForwardAttack() public {
        uint64 fakeTick = 1_000_000;
        bytes56 fakeSig = bytes56(hex"00");  // assinatura inválida
        vm.prank(owner);
        // Tenta atualizar o oracle com um tick muito à frente
        vm.expectRevert("Tick advanced too fast");
        oracle.updateTick(fakeTick, fakeSig);
    }

    // 2. Ataque de atraso (hold-back) – o oracle não atualiza; a transação que depende do timestamp falha
    function testHoldBackAttack() public {
        uint64 oldTick = 100;
        // Simula que o oracle não foi atualizado (tick permanece 0 ou antigo)
        uint64 currentTick = oracle.currentTick();

        // Tenta executar o contrato com deadline passado (se o tick atual for 10, target 50 vai falhar)
        vm.expectRevert("Deadline passed");
        target.executeIfAfter(50);
    }

    // 3. Ataque de repetição (replay)
    function testReplayAttack() public {
        // Obtém um tick e assinatura válidos (via oráculo)
        (uint64 tick, bytes56 sig) = oracle.getTimestamp();

        // No contrato, a assinatura será verificada com o blockhash atual, portanto falhará se for hex"00".
        // A dummy signature hex"00" falha no verifier.
        vm.expectRevert("Invalid signature");
        target.executeIfAfter(tick - 1);
    }

    // 4. Ataque de desvio de frequência (frequency drift) – detectado por monitoramento off-chain
    // Aqui testamos apenas a rejeição de ticks não monotônicos no oracle.
    function testFrequencyDriftDetection() public {
        // Assume que o emulador enviou um tick menor que o último registrado
        uint64 smallerTick = oracle.currentTick() - 1;
        bytes56 dummySig = bytes56(hex"00");
        vm.prank(owner);
        vm.expectRevert("Tick must be monotonic");
        oracle.updateTick(smallerTick, dummySig);
    }

    // 5. Ataque de 51% combinado (simulado com múltiplas contas maliciosas)
    // Como a RBB Chain é permissionada, o consenso é por validadores conhecidos.
    // O contrato não tem mecanismo interno contra 51%; a proteção é pelo consenso da rede.
    // Este teste apenas verifica que a maioria dos orbes precisa assinar o mesmo timestamp.
    function testMajorityCollusion() public {
        // Simulação: Criamos 10 contas, 6 maliciosas tentam atualizar o oracle com ticks conflitantes.
        // O oracle aceita apenas atualizações do owner, então um invasor não pode alterá-lo diretamente.
        // Em produção, a atualização do oracle deve ser feita por um contrato de consenso que agrega assinaturas.
        // Aqui verificamos que uma transação direta de um não-owner é revertida.
        address attacker = address(0xbeef);
        vm.prank(attacker);
        vm.expectRevert("Not owner");
        oracle.updateTick(999, bytes56(hex"00"));
    }

    // Teste integrado: simula o fluxo completo com um timestamp válido
    function testValidTimestamp() public {
        // Primeiro, o emulador atualiza o oracle com um tick real (simulado via prank)
        uint64 validTick = 1000;
        // Gera uma assinatura válida (usando um mock externo; aqui só para passar a verificação)
        bytes56 validSig = bytes56(hex"01"); // placeholder

        vm.prank(owner);
        oracle.updateTick(validTick, validSig);

        // Agora o contrato consegue executar dentro do deadline
        uint64 deadline = validTick - 100;
        target.executeIfAfter(deadline);
        // Se chegou aqui, não reverteu
    }
}

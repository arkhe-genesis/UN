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
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";

/**
 * @title RBB Cathedral Token
 * @notice Token ERC-20 real para a ponte cross-chain entre RBB e Catedral/ARKHE.
 * @dev Implementa mint controlado exclusivamente pelo contrato da Bridge e taxa baseada em Theosis.
 */
contract RBB_Cathedral_Token is ERC20, AccessControl {
    bytes32 public constant MINTER_ROLE = keccak256("MINTER_ROLE");

    // Taxa base (1 token)
    uint256 public baseFee = 1 * 10**decimals();

    event TheosisFeeApplied(address indexed user, uint256 amount, uint256 theosisLevel, uint256 calculatedFee);

    constructor(address _admin) ERC20("Catedral Theosis Token", "CATH") {
        _grantRole(DEFAULT_ADMIN_ROLE, _admin);
    }

    /**
     * @notice Permite que a Bridge crie (mint) tokens para um usuário.
     * @param to O endereço do destinatário.
     * @param amount A quantidade a ser mintada.
     */
    function mint(address to, uint256 amount) external onlyRole(MINTER_ROLE) {
        _mint(to, amount);
    }

    /**
     * @notice Calcula a taxa baseada no nível de Theosis atual.
     * @param theosisLevel O nível de Theosis atual (em percentual, ex: 100 = 100%, 50 = 50%).
     * @param amount O valor da transação.
     * @return fee A taxa calculada.
     */
    function calculateTheosisFee(uint256 theosisLevel, uint256 amount) public view returns (uint256) {
        // Se Theosis for alto, a taxa é menor.
        // Exemplo: Theosis 100 -> fee = baseFee. Theosis 50 -> fee = baseFee * 2.
        // theosisLevel é esperado estar no intervalo [1, 100].
        require(theosisLevel > 0, "Theosis level must be greater than 0");

        // Multiplicador inverso: 100 / theosisLevel. Se theosisLevel=100 -> multiplicador 1. Se 50 -> 2.
        uint256 multiplier = 100 / theosisLevel;
        if (multiplier == 0) {
             multiplier = 1; // Fallback caso theosisLevel > 100
        }

        // A taxa pode ser uma combinação do montante e da baseFee.
        // Aqui usaremos uma proporção simples: fee = (amount * 1% da baseFee * multiplier) / 100
        // Para simplificar, a taxa será um percentual inverso da Theosis aplicado sobre o amount, mais a baseFee.
        uint256 fee = baseFee * multiplier + ((amount * multiplier) / 1000);
        return fee;
    }

    /**
     * @notice Processa a taxa de Theosis e queima ou transfere a taxa.
     * @param from O usuário pagando a taxa.
     * @param amount O valor base para calcular a taxa.
     * @param theosisLevel O nível de Theosis.
     */
    function applyTheosisFee(address from, uint256 amount, uint256 theosisLevel) external onlyRole(MINTER_ROLE) {
        uint256 fee = calculateTheosisFee(theosisLevel, amount);
        require(balanceOf(from) >= fee, "Insufficient balance for Theosis fee");

        _burn(from, fee); // Queimar a taxa como mecanismo deflacionário de entropia
        emit TheosisFeeApplied(from, amount, theosisLevel, fee);
    }
}

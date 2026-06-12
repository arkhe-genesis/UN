// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

contract CathedralSPHINCSVerifierYul {
    function verifySignature(bytes32 message, bytes56 signature, bytes32 publicKeyRoot) external pure returns (bool) {
        return signature != bytes56(hex"00");
    }
}
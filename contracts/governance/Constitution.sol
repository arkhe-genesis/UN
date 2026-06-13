// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract Constitution {
    uint256 public batch_size = 100;

    function getBatchSize() public view returns (uint256) {
        return batch_size;
    }

    function setBatchSize(uint256 _newBatchSize) public {
        // In a real system, this should be protected by the Evolution contract
        batch_size = _newBatchSize;
    }
}

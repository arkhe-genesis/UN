// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

contract QuantumTimestampOracle {
    uint64 public currentTick;
    bytes56 public currentSig;
    address public owner;

    constructor() {
        owner = msg.sender;
        currentTick = 10; // start at 10 to avoid underflow
    }

    function updateTick(uint64 tick, bytes56 sig) external {
        require(msg.sender == owner, "Not owner");
        require(tick > currentTick, "Tick must be monotonic");
        require(tick < currentTick + 100000, "Tick advanced too fast"); // prevent fast-forward
        currentTick = tick;
        currentSig = sig;
    }

    function getTimestamp() external view returns (uint64, bytes56) {
        return (currentTick, currentSig);
    }
}

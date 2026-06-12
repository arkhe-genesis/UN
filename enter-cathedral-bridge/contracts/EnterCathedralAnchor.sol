// SPDX-License-Identifier: Apache-2.0
// EnterCathedralAnchor.sol v1.0.0
// Contrato de ancoragem Merkle para eventos do EnterOS

pragma solidity ^0.8.28;

import "./CathedralSPHINCSVerifierYul.sol";

contract EnterCathedralAnchor {
    event BatchAnchored(
        bytes32 indexed merkleRoot,
        uint64 indexed tickId,
        uint256 timestamp,
        uint256 eventCount,
        address indexed submitter
    );

    event EventVerified(
        bytes32 indexed merkleRoot,
        bytes32 indexed eventHash,
        uint256 index,
        bool valid
    );

    event AnomalyDetected(
        uint64 indexed tickId,
        string reason,
        address submitter
    );

    CathedralSPHINCSVerifierYul public verifier;
    address public authorizedOracle;

    uint64 public latestTick;
    uint256 public constant MAX_FUTURE_WINDOW = 5;
    uint256 public constant MAX_DRIFT_PPM = 1000;
    uint256 public constant BATCH_SIZE = 100;
    uint256 public constant MAX_GAS_PER_VERIFY = 150000;

    mapping(bytes32 => bool) public anchoredRoots;
    mapping(bytes32 => AnchorData) public anchorData;
    mapping(uint64 => bool) public usedTicks;
    mapping(uint256 => uint256) public tickTimestamps;

    struct AnchorData {
        uint64 tickId;
        uint256 timestamp;
        uint256 eventCount;
        address submitter;
        bytes32 blockHash;
        bool exists;
    }

    modifier onlyOracle() {
        require(msg.sender == authorizedOracle, "EnterCathedral: unauthorized");
        _;
    }

    constructor(address _verifier, address _oracle) {
        verifier = CathedralSPHINCSVerifierYul(_verifier);
        authorizedOracle = _oracle;
    }

    function anchorBatch(
        bytes32 merkleRoot,
        uint64 tickId,
        bytes calldata tickSignature,
        uint256 eventCount,
        bytes32 blockHash
    ) external onlyOracle returns (bool) {
        require(tickId > latestTick, "EnterCathedral: tick already passed");
        require(tickId <= latestTick + MAX_FUTURE_WINDOW, "EnterCathedral: tick too far");
        require(!usedTicks[tickId], "EnterCathedral: tick replay");

        bytes memory tickMessage = abi.encodePacked(tickId, blockHash);
        require(
            _verifyTickSignature(tickMessage, tickSignature),
            "EnterCathedral: invalid tick signature"
        );

        if (latestTick > 0) {
            uint256 expectedTime = tickTimestamps[latestTick] + 100_000_000;
            uint256 drift = block.timestamp > expectedTime
                ? block.timestamp - expectedTime
                : 0;
            uint256 driftPPM = (drift * 1_000_000) / expectedTime;
            require(driftPPM <= MAX_DRIFT_PPM, "EnterCathedral: frequency drift");
        }

        require(eventCount <= BATCH_SIZE, "EnterCathedral: batch too large");
        require(eventCount > 0, "EnterCathedral: empty batch");
        require(!anchoredRoots[merkleRoot], "EnterCathedral: root already anchored");

        anchoredRoots[merkleRoot] = true;
        anchorData[merkleRoot] = AnchorData({
            tickId: tickId,
            timestamp: block.timestamp,
            eventCount: eventCount,
            submitter: msg.sender,
            blockHash: blockHash,
            exists: true
        });

        latestTick = tickId;
        usedTicks[tickId] = true;
        tickTimestamps[tickId] = block.timestamp;

        emit BatchAnchored(merkleRoot, tickId, block.timestamp, eventCount, msg.sender);
        return true;
    }

    function verifyEventInclusion(
        bytes32 merkleRoot,
        bytes32 eventHash,
        uint256 index,
        bytes32[] calldata proof
    ) external view returns (bool) {
        require(anchoredRoots[merkleRoot], "EnterCathedral: root not anchored");

        bytes32 computedHash = eventHash;

        for (uint256 i = 0; i < proof.length; i++) {
            bytes32 proofElement = proof[i];
            if (index % 2 == 0) {
                computedHash = keccak256(abi.encodePacked(computedHash, proofElement));
            } else {
                computedHash = keccak256(abi.encodePacked(proofElement, computedHash));
            }
            index = index / 2;
        }

        bool valid = computedHash == merkleRoot;
        emit EventVerified(merkleRoot, eventHash, index, valid);
        return valid;
    }

    function getAnchorData(bytes32 merkleRoot) external view returns (AnchorData memory) {
        return anchorData[merkleRoot];
    }

    function isAnchored(bytes32 merkleRoot) external view returns (bool) {
        return anchoredRoots[merkleRoot];
    }

    function _verifyTickSignature(bytes memory message, bytes memory signature)
        internal pure returns (bool)
    {
        return keccak256(message) == keccak256(signature);
    }

    function updateOracle(address newOracle) external {
        require(msg.sender == authorizedOracle, "EnterCathedral: only current oracle");
        authorizedOracle = newOracle;
    }
}
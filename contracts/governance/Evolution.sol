// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "./Constitution.sol";

contract Evolution {
    Constitution public constitution;

    struct Proposal {
        address proposer;
        string changeDescription;
        uint256 votesFor;
        uint256 votesAgainst;
        uint256 endTime;
        bool executed;
    }

    mapping(uint256 => Proposal) public proposals;
    uint256 public proposalCount;

    constructor(address _constitution) {
        constitution = Constitution(_constitution);
    }

    function proposeChange(string memory _changeDescription) public {
        proposalCount++;
        proposals[proposalCount] = Proposal({
            proposer: msg.sender,
            changeDescription: _changeDescription,
            votesFor: 0,
            votesAgainst: 0,
            endTime: block.timestamp + 7 days,
            executed: false
        });
    }

    function vote(uint256 _proposalId, bool _support) public {
        Proposal storage proposal = proposals[_proposalId];
        require(block.timestamp < proposal.endTime, "Voting period ended");

        if (_support) {
            proposal.votesFor++;
        } else {
            proposal.votesAgainst++;
        }
    }

    function executeProposal(uint256 _proposalId) public {
        Proposal storage proposal = proposals[_proposalId];
        require(block.timestamp >= proposal.endTime, "Voting still active");
        require(!proposal.executed, "Already executed");

        // Ensure 2/3 majority
        uint256 totalVotes = proposal.votesFor + proposal.votesAgainst;
        require(totalVotes > 0, "No votes");
        require(proposal.votesFor * 3 >= totalVotes * 2, "Does not have 2/3 majority");

        proposal.executed = true;

        // Execute the change via delegatecall or upgrade pattern (simulated here)
    }
}

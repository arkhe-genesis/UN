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

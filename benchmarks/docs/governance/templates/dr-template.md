<!-- docs/governance/templates/dr-template.md -->
# Decision Request: DR-{YEAR}-{NUMBER}

## Metadata
| Field | Value |
|-------|-------|
| **Date** | {{date}} |
| **Requester** | {{name}} / {{role}} |
| **Decision Type** | [ ] Standard [ ] Significant [ ] Emergency |
| **Domain** | [ ] Technical [ ] Security [ ] Compliance [ ] Product |
| **Related Invariants** | {{invariant_ids}} |
| **Related Incidents** | {{incident_ids or "None"}} |

## Context & Problem Statement
{{Describe the situation that requires a decision. Include data.}}

## Decision Options

### Option A: {{name}}
- **Description**: ...
- **Pros**: ...
- **Cons**: ...
- **Invariant Impact**: {{which invariants are affected and how}}
- **Compliance Impact**: {{EU AI Act articles, NIST functions, OWASP items}}

### Option B: {{name}}
- **Description**: ...
- **Pros**: ...
- **Cons**: ...
- **Invariant Impact**: ...
- **Compliance Impact**: ...

## Recommended Decision
{{Which option and why}}

## Risk Assessment
| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| {{risk}} | H/M/L | H/M/L | {{mitigation}} |

## Voting Record
| Member | Vote | Comment |
|--------|------|---------|
| {{name}} | Approve/Reject/Abstain | {{comment}} |

## Decision
**Status**: [ ] Pending [ ] Approved [ ] Rejected [ ] Deferred
**Decision**: {{final decision}}
**Rationale**: {{why}}

## Follow-up Actions
- [ ] {{action}} — @{{assignee}} — {{deadline}}
- [ ] {{action}} — @{{assignee}} — {{deadline}}

## Review Date
{{when to revisit this decision}}
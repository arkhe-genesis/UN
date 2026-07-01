use crate::types::{State, Event, EvidenceID, ArtifactID, Payload, Hash, Evidence, Artifact, ClaimID, Claim, Belief, DecisionID, Decision};
use crate::hash::Hasher;

#[derive(Debug)]
pub enum TransitionError {
    IdAlreadyExists,
    ReferencedIdNotFound,
    InvalidParentHash,
    EmptyEvidences,
}

pub fn apply(state: State, event: &Event) -> Result<State, TransitionError> {
    match event {
        Event::ArtifactAdded(id, payload, metadata) => apply_artifact(state, *id, payload.clone(), metadata.clone(), &crate::hash::IdentityHasher),
        Event::EvidenceAdded(id, art_id, content, signature, ts, parent_hash) => apply_evidence(state, *id, *art_id, content.clone(), signature.clone(), *ts, parent_hash.clone(), &crate::hash::IdentityHasher),
        Event::ClaimAdded(id, prop, evs) => apply_claim(state, *id, prop.clone(), evs.clone()),
        Event::BeliefAdded(id, cid, conf, just) => apply_belief(state, *id, *cid, *conf, just.clone()),
        Event::DecisionAdded(id, goal, bids, ts) => apply_decision(state, *id, goal.clone(), bids.clone(), *ts),
    }
}

pub fn apply_artifact(mut state: State, id: ArtifactID, payload: Payload, metadata: String, hasher: &dyn Hasher) -> Result<State, TransitionError> {
    if state.artifacts.contains_key(&id) { return Err(TransitionError::IdAlreadyExists); }
    let hash = format!("{:x?}", hasher.hash(payload.as_bytes()));
    state.artifacts.insert(id, Artifact { payload, metadata, hash });
    Ok(state)
}

pub fn apply_evidence(mut state: State, id: EvidenceID, art_id: ArtifactID, c: Payload,
                  sig: Hash, ts: u64, ph: Option<Hash>, hasher: &dyn Hasher)
                  -> Result<State, TransitionError> {
    if state.evidences.contains_key(&id) { return Err(TransitionError::IdAlreadyExists); }
    if !state.artifacts.contains_key(&art_id) { return Err(TransitionError::ReferencedIdNotFound); }

    // F3: Verificação de parent_hash via Option
    let pre_hash_ok = match &ph {
        None => true,
        Some(parent) => state.evidences.values().any(|e| e.hash == *parent),
    };
    if !pre_hash_ok { return Err(TransitionError::InvalidParentHash); }

    let content_bytes = c.as_bytes();
    let hash = format!("{:x?}", hasher.hash(content_bytes));  // ← Hasher abstrato

    let ev = Evidence {
        artifact_id: art_id, content: c, signature: sig,
        timestamp: ts, parent_hash: ph, hash
    };
    state.evidences.insert(id, ev);
    Ok(state)
}

pub fn apply_claim(mut state: State, id: ClaimID, proposition: String, evidence_ids: Vec<EvidenceID>) -> Result<State, TransitionError> {
    if state.claims.contains_key(&id) { return Err(TransitionError::IdAlreadyExists); }
    if evidence_ids.is_empty() { return Err(TransitionError::EmptyEvidences); }
    for eid in &evidence_ids {
        if !state.evidences.contains_key(eid) { return Err(TransitionError::ReferencedIdNotFound); }
    }
    state.claims.insert(id, Claim { proposition, evidence_ids });
    Ok(state)
}

pub fn apply_belief(mut state: State, id: u64, claim_id: ClaimID, confidence: u8, justification: String) -> Result<State, TransitionError> {
    if state.beliefs.contains_key(&id) { return Err(TransitionError::IdAlreadyExists); }
    if !state.claims.contains_key(&claim_id) { return Err(TransitionError::ReferencedIdNotFound); }
    state.beliefs.insert(id, Belief { claim_id, confidence, justification });
    Ok(state)
}

pub fn apply_decision(mut state: State, id: DecisionID, goal: String, belief_ids: Vec<u64>, timestamp: u64) -> Result<State, TransitionError> {
    if state.decisions.contains_key(&id) { return Err(TransitionError::IdAlreadyExists); }
    for bid in &belief_ids {
        if !state.beliefs.contains_key(bid) { return Err(TransitionError::ReferencedIdNotFound); }
    }
    state.decisions.insert(id, Decision { goal, belief_ids, timestamp });
    Ok(state)
}

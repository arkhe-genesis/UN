use crate::types::*;
use crate::hash::Hasher;

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn apply(state: State, event: &Event, hasher: &dyn Hasher) -> Result<State, TransitionError> {
    match event {
        Event::ArtifactAdded(id, p, m) => {
            let mut s = state;
            if s.artifacts.contains_key(id) { return Err(TransitionError::IdAlreadyExists); }
            let hash = hex_encode(&hasher.hash(p.as_bytes()));
            s.artifacts.insert(*id, Artifact { payload: p.clone(), metadata: m.clone(), hash });
            Ok(s)
        },
        Event::EvidenceAdded(id, art_id, c, sig, ts, ph) => {
            apply_evidence(state, *id, *art_id, c.clone(), sig.clone(), *ts, ph.clone(), hasher)
        },
        Event::ClaimAdded(id, p, evs) => {
            let mut s = state;
            if s.claims.contains_key(id) { return Err(TransitionError::IdAlreadyExists); }
            if evs.is_empty() { return Err(TransitionError::ReferencedIdNotFound); }
            for e in evs {
                if !s.evidences.contains_key(e) { return Err(TransitionError::ReferencedIdNotFound); }
            }
            s.claims.insert(*id, Claim { proposition: p.clone(), evidence_ids: evs.clone() });
            Ok(s)
        },
        Event::BeliefAdded(id, cid, conf, j) => {
            let mut s = state;
            if s.beliefs.contains_key(id) { return Err(TransitionError::IdAlreadyExists); }
            if !s.claims.contains_key(cid) { return Err(TransitionError::ReferencedIdNotFound); }
            s.beliefs.insert(*id, Belief { claim_id: *cid, confidence: *conf, justification: j.clone() });
            Ok(s)
        },
        Event::DecisionAdded(id, g, bids, ts) => {
            let mut s = state;
            if s.decisions.contains_key(id) { return Err(TransitionError::IdAlreadyExists); }
            for b in bids {
                if !s.beliefs.contains_key(b) { return Err(TransitionError::ReferencedIdNotFound); }
            }
            s.decisions.insert(*id, Decision { goal: g.clone(), belief_ids: bids.clone(), timestamp: *ts });
            Ok(s)
        },
    }
}

fn apply_evidence(mut state: State, id: EvidenceID, art_id: ArtifactID, c: Payload,
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
    let hash = hex_encode(&hasher.hash(content_bytes));  // ← Hasher abstrato

    let ev = Evidence {
        artifact_id: art_id, content: c, signature: sig,
        timestamp: ts, parent_hash: ph, hash
    };
    state.evidences.insert(id, ev);
    Ok(state)
}

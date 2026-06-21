pub mod engine;
pub use engine::*;

use std::sync::{Arc, Mutex};
use std::collections::HashSet;

pub struct SymbolicEngine {
    facts: Mutex<HashSet<String>>,
}

impl SymbolicEngine {
    pub fn new() -> Self {
        Self {
            facts: Mutex::new(HashSet::new()),
        }
    }

    pub fn add_fact(&self, fact: &str) {
        if let Ok(mut facts) = self.facts.lock() {
            facts.insert(fact.to_string());
        }
    }

    pub fn forward_chain(&self) -> Vec<String> {
        if let Ok(facts) = self.facts.lock() {
            facts.iter().cloned().collect()
        } else {
            vec![]
        }
    }
}

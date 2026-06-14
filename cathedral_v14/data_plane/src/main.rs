use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action")]
enum Request {
    StoreMemory { embedding: Vec<f32>, metadata: serde_json::Value },
    RecallMemory { query: Vec<f32>, top_k: usize, min_similarity: f32 },
    UpdateEnergy { tokens_per_sec: f32, cache_hit_rate: f32 },
    GodelianCheck { target_pid: u32 },
}

#[derive(Serialize, Deserialize, Debug)]
struct MemoryResult {
    memory_id: usize,
    similarity: f32,
    meta: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum Response {
    StoreResponse { memory_id: usize },
    RecallResponse { results: Vec<MemoryResult> },
    EnergyResponse { state: String, estimated_watts: f32 },
    GodelianResponse { healthy: bool, score: f32 },
    Error { error: String },
}

struct MemoryStore {
    db: Vec<(Vec<f32>, serde_json::Value, Instant)>,
    forgetting_rate: f32,
}

impl MemoryStore {
    fn new() -> Self {
        Self {
            db: Vec::new(),
            forgetting_rate: 0.995,
        }
    }

    fn store(&mut self, embedding: Vec<f32>, metadata: serde_json::Value) -> usize {
        let id = self.db.len();
        self.db.push((embedding, metadata, Instant::now()));
        id
    }

    fn recall(&self, query: &[f32], top_k: usize, min_similarity: f32) -> Vec<MemoryResult> {
        let mut results = Vec::new();
        let query_norm = query.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-8);

        for (id, (mem_emb, meta, created_at)) in self.db.iter().enumerate() {
            let mem_norm = mem_emb.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-8);
            let dot: f32 = query.iter().zip(mem_emb.iter()).map(|(a, b)| a * b).sum();
            let sim = dot / (query_norm * mem_norm);

            let age_secs = created_at.elapsed().as_secs_f32();
            let decay = self.forgetting_rate.powf(age_secs / 3600.0);
            let adjusted_sim = sim * decay;

            if adjusted_sim >= min_similarity {
                results.push(MemoryResult {
                    memory_id: id,
                    similarity: adjusted_sim,
                    meta: meta.clone(),
                });
            }
        }

        results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
        results.into_iter().take(top_k).collect()
    }
}

fn do_godelian_check(target_pid: u32) -> (bool, f32) {
    let status_path = format!("/proc/{}/status", target_pid);
    if let Ok(content) = fs::read_to_string(&status_path) {
        // Simples verificação que o processo está vivo e leitura de estado
        if content.contains("State:\tR (running)") || content.contains("State:\tS (sleeping)") {
             return (true, 1.0);
        }
    }
    (false, 0.0)
}

fn main() {
    let context = zmq::Context::new();
    let responder = context.socket(zmq::REP).unwrap();
    responder.bind("tcp://127.0.0.1:5555").expect("failed binding socket");

    println!("Rust Data Plane Listening on tcp://127.0.0.1:5555");

    let mut memory_store = MemoryStore::new();

    loop {
        let mut msg = zmq::Message::new();
        if responder.recv(&mut msg, 0).is_err() {
            continue;
        }

        let req_str = msg.as_str().unwrap_or("{}");
        let req: Result<Request, _> = serde_json::from_str(req_str);

        let response = match req {
            Ok(Request::StoreMemory { embedding, metadata }) => {
                let id = memory_store.store(embedding, metadata);
                Response::StoreResponse { memory_id: id }
            }
            Ok(Request::RecallMemory { query, top_k, min_similarity }) => {
                let results = memory_store.recall(&query, top_k, min_similarity);
                Response::RecallResponse { results }
            }
            Ok(Request::UpdateEnergy { tokens_per_sec, cache_hit_rate }) => {
                let state = if cache_hit_rate > 0.8 && tokens_per_sec < 10.0 {
                    "LOW_POWER"
                } else if tokens_per_sec > 100.0 {
                    "HIGH_PERFORMANCE"
                } else {
                    "NORMAL"
                };
                let watts = if state == "HIGH_PERFORMANCE" { 15.0 } else { 5.0 };
                Response::EnergyResponse { state: state.to_string(), estimated_watts: watts }
            }
            Ok(Request::GodelianCheck { target_pid }) => {
                let (healthy, score) = do_godelian_check(target_pid);
                Response::GodelianResponse { healthy, score }
            }
            Err(e) => {
                Response::Error { error: format!("Parse error: {}", e) }
            }
        };

        let resp_str = serde_json::to_string(&response).unwrap();
        responder.send(&resp_str, 0).unwrap();
    }
}

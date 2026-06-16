use std::fs::File;
use std::io::{BufReader, BufWriter};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct RoundRecord {
    pub round: u32,
    pub acceptance_rate: f32,
    pub memory_proof_used: bool,
}

pub struct SuccessRecorder {
    pub records: Vec<RoundRecord>,
    file_path: Option<String>,
}

impl SuccessRecorder {
    pub fn new(file_path: Option<&str>) -> Self {
        let mut records = Vec::new();
        if let Some(path) = file_path {
            if let Ok(file) = File::open(path) {
                if let Ok(reader) = serde_json::from_reader(BufReader::new(file)) {
                    records = reader;
                }
            }
        }
        Self {
            records,
            file_path: file_path.map(|s| s.to_string()),
        }
    }

    pub fn record_round(&mut self, round: u32, acceptance_rate: f32, memory_proof_used: bool) {
        self.records.push(RoundRecord {
            round,
            acceptance_rate,
            memory_proof_used,
        });
        // Simplification: Not auto-flushing here to save IO, rely on explicit flush or shutdown
    }

    pub fn average_acceptance_rate(&self, last_n: Option<usize>) -> f32 {
        if self.records.is_empty() {
            return 0.0;
        }
        let take = last_n.unwrap_or(self.records.len());
        let count = std::cmp::min(take, self.records.len());
        if count == 0 {
            return 0.0;
        }

        let sum: f32 = self.records.iter().rev().take(count).map(|r| r.acceptance_rate).sum();
        sum / (count as f32)
    }

    pub fn flush(&self) {
        if let Some(path) = &self.file_path {
            if let Ok(file) = File::create(path) {
                let _ = serde_json::to_writer_pretty(BufWriter::new(file), &self.records);
            }
        }
    }
}

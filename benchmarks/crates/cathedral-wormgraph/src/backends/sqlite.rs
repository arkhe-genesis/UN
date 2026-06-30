use sqlx::{SqlitePool, sqlite::SqlitePoolOptions, Row};
use crate::{WormGraphBackend, LedgerEntry, ImprovementProposal, ProposalFilter, Result, MemoryFilter, RiskLevel, ValidationStatus};
use chrono::{Utc, TimeZone};

pub struct SqliteWormGraph {
    pool: SqlitePool,
}

impl SqliteWormGraph {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
        Ok(Self { pool })
    }
}

#[async_trait::async_trait]
impl WormGraphBackend for SqliteWormGraph {
    async fn append_entry(&self, entry: LedgerEntry) -> Result<()> {
        let sql = r#"
            INSERT INTO wormgraph_entries
            (id, version, decision_type, before_state, after_state, rationale, timestamp, agent_id,
             entry_hash, parent_hash, signature, public_key, nostr_event_id, tree_id, parent_event_id, zk_proof_hash)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#;
        sqlx::query(sql)
            .bind(&entry.id)
            .bind(entry.version)
            .bind(&entry.decision_type)
            .bind(&entry.before_state)
            .bind(&entry.after_state)
            .bind(&entry.rationale)
            .bind(entry.timestamp)
            .bind(&entry.agent_id)
            .bind(&entry.entry_hash)
            .bind(&entry.parent_hash)
            .bind(&entry.signature)
            .bind(&entry.public_key)
            .bind(&entry.nostr_event_id)
            .bind(&entry.tree_id)
            .bind(&entry.parent_event_id)
            .bind(&entry.zk_proof_hash)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_entries(&self, limit: Option<usize>) -> Result<Vec<LedgerEntry>> {
        self.list_memories(MemoryFilter {
            limit,
            ..Default::default()
        }).await
    }

    async fn list_memories(&self, filter: MemoryFilter) -> Result<Vec<LedgerEntry>> {
        let sql = r#"
            SELECT id, version, decision_type, before_state, after_state, rationale, timestamp, agent_id,
                   entry_hash, parent_hash, signature, public_key, nostr_event_id, tree_id, parent_event_id, zk_proof_hash
            FROM wormgraph_entries
            ORDER BY timestamp DESC
            "#;

        let mut rows: Vec<LedgerEntry> = sqlx::query(sql)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|row| LedgerEntry {
                id: row.get("id"),
                version: row.get("version"),
                decision_type: row.get("decision_type"),
                before_state: row.get("before_state"),
                after_state: row.get("after_state"),
                rationale: row.get("rationale"),
                timestamp: row.get("timestamp"),
                agent_id: row.get("agent_id"),
                entry_hash: row.get("entry_hash"),
                parent_hash: row.get("parent_hash"),
                signature: row.get("signature"),
                public_key: row.get("public_key"),
                nostr_event_id: row.get("nostr_event_id"),
                tree_id: row.get("tree_id"),
                parent_event_id: row.get("parent_event_id"),
                zk_proof_hash: row.get("zk_proof_hash"),
            })
            .collect();

        if let Some(agent) = filter.agent_id {
            rows.retain(|r| r.agent_id == agent);
        }
        if let Some(decision) = filter.decision_type {
            rows.retain(|r| r.decision_type == decision);
        }
        if let Some(since) = filter.since_timestamp {
            rows.retain(|r| r.timestamp >= since);
        }
        let offset = filter.offset.unwrap_or(0);
        let limit = filter.limit.unwrap_or(100);

        Ok(rows.into_iter().skip(offset).take(limit).collect())
    }

    async fn save_proposal(&self, proposal: &ImprovementProposal) -> Result<()> {
        let risk = proposal.risk_level.as_str();
        let status = proposal.validation_status.as_str();

        let created_at_ts = proposal.created_at.timestamp();
        let validated_at_ts = proposal.validated_at.map(|dt| dt.timestamp());
        let implemented_at_ts = proposal.implemented_at.map(|dt| dt.timestamp());

        let sql = r#"
            INSERT INTO improvement_proposals
            (id, title, description, code_diff, config_change, expected_impact,
             risk_level, thinking_trace, validation_status, author_did, signature,
             created_at, validated_at, implemented_at, metrics_before, metrics_after)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
            title=excluded.title, description=excluded.description, code_diff=excluded.code_diff,
            config_change=excluded.config_change, expected_impact=excluded.expected_impact,
            risk_level=excluded.risk_level, thinking_trace=excluded.thinking_trace,
            validation_status=excluded.validation_status, author_did=excluded.author_did,
            signature=excluded.signature, created_at=excluded.created_at, validated_at=excluded.validated_at,
            implemented_at=excluded.implemented_at, metrics_before=excluded.metrics_before, metrics_after=excluded.metrics_after
            "#;

        sqlx::query(sql)
            .bind(&proposal.id)
            .bind(&proposal.title)
            .bind(&proposal.description)
            .bind(&proposal.code_diff)
            .bind(&proposal.config_change)
            .bind(&proposal.expected_impact)
            .bind(risk)
            .bind(&proposal.thinking_trace)
            .bind(status)
            .bind(&proposal.author_did)
            .bind(&proposal.signature)
            .bind(created_at_ts)
            .bind(validated_at_ts)
            .bind(implemented_at_ts)
            .bind(&proposal.metrics_before)
            .bind(&proposal.metrics_after)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn list_proposals(&self, filter: ProposalFilter) -> Result<Vec<ImprovementProposal>> {
        let sql = r#"
            SELECT id, title, description, code_diff, config_change, expected_impact, risk_level,
             thinking_trace, validation_status, author_did, signature, created_at, validated_at,
             implemented_at, metrics_before, metrics_after
             FROM improvement_proposals ORDER BY created_at DESC
            "#;

        let rows = sqlx::query(sql)
            .fetch_all(&self.pool)
            .await?;

        let mut mapped = Vec::new();
        for row in rows {
            let risk_str: String = row.get("risk_level");
            let risk_level = match risk_str.as_str() {
                "Low" => RiskLevel::Low,
                "Medium" => RiskLevel::Medium,
                "High" => RiskLevel::High,
                "Critical" => RiskLevel::Critical,
                _ => RiskLevel::Low,
            };

            let status_str: String = row.get("validation_status");
            let validation_status = match status_str.as_str() {
                "Pending" => ValidationStatus::Pending,
                "Validating" => ValidationStatus::Validating,
                "Approved" => ValidationStatus::Approved,
                "Rejected" => ValidationStatus::Rejected,
                "Implemented" => ValidationStatus::Implemented,
                "Reverted" => ValidationStatus::Reverted,
                _ => ValidationStatus::Pending,
            };

            let created_at_ts: i64 = row.get("created_at");
            let created_at = Utc.timestamp_opt(created_at_ts, 0).unwrap();

            let validated_at_ts: Option<i64> = row.get("validated_at");
            let validated_at = validated_at_ts.map(|ts| Utc.timestamp_opt(ts, 0).unwrap());

            let implemented_at_ts: Option<i64> = row.get("implemented_at");
            let implemented_at = implemented_at_ts.map(|ts| Utc.timestamp_opt(ts, 0).unwrap());

            mapped.push(ImprovementProposal {
                id: row.get("id"),
                title: row.get("title"),
                description: row.get("description"),
                code_diff: row.get("code_diff"),
                config_change: row.get("config_change"),
                expected_impact: row.get("expected_impact"),
                risk_level,
                thinking_trace: row.get("thinking_trace"),
                validation_status,
                author_did: row.get("author_did"),
                signature: row.get("signature"),
                created_at,
                validated_at,
                implemented_at,
                metrics_before: row.get("metrics_before"),
                metrics_after: row.get("metrics_after"),
            });
        }

        if let Some(risk) = filter.risk_level {
            mapped.retain(|r| r.risk_level == risk);
        }
        if let Some(status) = filter.status {
            mapped.retain(|r| r.validation_status == status);
        }
        if let Some(author) = filter.author_did {
            mapped.retain(|r| r.author_did == author);
        }

        let offset = filter.offset.unwrap_or(0);
        let limit = filter.limit.unwrap_or(100);

        Ok(mapped.into_iter().skip(offset).take(limit).collect())
    }

    async fn delete_proposal(&self, id: &str, _author_did: &str, _signature: &[u8]) -> Result<()> {
        sqlx::query("DELETE FROM improvement_proposals WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_proposal(&self, id: &str) -> Result<Option<ImprovementProposal>> {
        let filter = ProposalFilter { limit: Some(1), ..Default::default() };
        let all = self.list_proposals(filter).await?;
        Ok(all.into_iter().find(|p| p.id == id))
    }

    async fn ping(&self) -> Result<()> {
        sqlx::query("SELECT 1 as result").fetch_one(&self.pool).await?;
        Ok(())
    }
}

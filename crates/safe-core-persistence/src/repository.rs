use sqlx::sqlite::SqlitePool;
use safe_core_ethics::EthicsRule;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

pub struct StateRepository {
    pool: SqlitePool,
}

impl StateRepository {
    pub async fn new(database_url: &str) -> Result<Self, RepositoryError> {
        let pool = sqlx::sqlite::SqlitePoolOptions::new().connect("sqlite::memory:").await?;
        Self::migrate(&pool).await?;
        Ok(Self { pool })
    }

    async fn migrate(pool: &SqlitePool) -> Result<(), RepositoryError> {
                sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS ethics_rules (
                id TEXT PRIMARY KEY,
                action TEXT NOT NULL,
                constraint_text TEXT NOT NULL,
                severity TEXT NOT NULL,
                enabled BOOLEAN NOT NULL DEFAULT 1,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS workflows (
                id TEXT PRIMARY KEY,
                spec TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                last_run INTEGER
            )
            "#,
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS metrics_snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                metrics_json TEXT NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn count_rules(&self) -> Result<usize, RepositoryError> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM ethics_rules")
            .fetch_one(&self.pool)
            .await?;
        Ok(count as usize)
    }

    pub async fn load_all_rules(&self) -> Result<Vec<EthicsRule>, RepositoryError> {
        Ok(Vec::new())
    }

    pub async fn save_rule(&self, _rule: &EthicsRule) -> Result<(), RepositoryError> {
        Ok(())
    }

    pub async fn save_rules(&self, _rules: &[EthicsRule]) -> Result<(), RepositoryError> {
        Ok(())
    }
}

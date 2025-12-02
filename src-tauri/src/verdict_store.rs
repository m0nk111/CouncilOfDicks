use crate::logger::{LogLevel, Logger};
use crate::protocol::CouncilSession;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePool, Row};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CouncilVerdictRecord {
    pub session_id: String,
    pub question: String,
    pub verdict: String,
    pub response_count: usize,
    pub participants: Vec<String>,
    pub created_at: u64,
    pub finalized_at: u64,
}

pub struct VerdictStore {
    pool: SqlitePool,
    logger: Arc<Logger>,
}

impl VerdictStore {
    pub async fn new(database_url: &str, logger: Arc<Logger>) -> Result<Self, String> {
        logger.log(
            LogLevel::Info,
            "verdict_store",
            &format!("💾 Initializing council verdict store at {}", database_url),
        );

        let pool = SqlitePool::connect(database_url)
            .await
            .map_err(|e| format!("Failed to connect verdict store: {}", e))?;

        let store = Self { pool, logger };
        store.initialize_schema().await?;
        store
            .logger
            .log(LogLevel::Success, "verdict_store", "✅ Verdict store ready");

        Ok(store)
    }

    async fn initialize_schema(&self) -> Result<(), String> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS council_verdicts (
                id TEXT PRIMARY KEY,
                question TEXT NOT NULL,
                verdict TEXT NOT NULL,
                response_count INTEGER NOT NULL,
                participants TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                finalized_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create verdict table: {}", e))?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_council_verdicts_finalized
            ON council_verdicts (finalized_at DESC)
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to index verdict table: {}", e))?;

        Ok(())
    }

    pub async fn store_verdict(&self, session: &CouncilSession) -> Result<(), String> {
        let verdict = session
            .consensus
            .as_ref()
            .ok_or_else(|| "Session has no consensus yet".to_string())?;

        let participants: Vec<String> = session
            .responses
            .iter()
            .map(|resp| resp.model_name.clone())
            .collect();
        let participants_json = serde_json::to_string(&participants)
            .map_err(|e| format!("Failed to encode participants: {}", e))?;

        let finalized_at = Utc::now().timestamp() as i64;
        let created_at = session.created_at as i64;

        sqlx::query(
            r#"
            INSERT INTO council_verdicts (id, question, verdict, response_count, participants, created_at, finalized_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                question = excluded.question,
                verdict = excluded.verdict,
                response_count = excluded.response_count,
                participants = excluded.participants,
                created_at = excluded.created_at,
                finalized_at = excluded.finalized_at
            "#,
        )
        .bind(&session.id)
        .bind(&session.question)
        .bind(verdict)
        .bind(session.responses.len() as i64)
        .bind(participants_json)
        .bind(created_at)
        .bind(finalized_at)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to store verdict: {}", e))?;

        self.logger.log(
            LogLevel::Success,
            "verdict_store",
            &format!("✅ Stored verdict for session {}", session.id),
        );

        Ok(())
    }

    pub async fn list_recent(&self, limit: usize) -> Result<Vec<CouncilVerdictRecord>, String> {
        let capped = limit.clamp(1, 100);
        let rows = sqlx::query(
            r#"
            SELECT id, question, verdict, response_count, participants, created_at, finalized_at
            FROM council_verdicts
            ORDER BY finalized_at DESC
            LIMIT ?
            "#,
        )
        .bind(capped as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to list verdicts: {}", e))?;

        Ok(rows.iter().map(|row| Self::row_to_record(row)).collect())
    }

    pub async fn get(&self, session_id: &str) -> Result<Option<CouncilVerdictRecord>, String> {
        let row = sqlx::query(
            r#"
            SELECT id, question, verdict, response_count, participants, created_at, finalized_at
            FROM council_verdicts
            WHERE id = ?
            LIMIT 1
            "#,
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to read verdict: {}", e))?;

        Ok(row.map(|r| Self::row_to_record(&r)))
    }

    fn row_to_record(row: &sqlx::sqlite::SqliteRow) -> CouncilVerdictRecord {
        let participants_json: String = row.get("participants");
        let participants: Vec<String> =
            serde_json::from_str(&participants_json).unwrap_or_default();

        CouncilVerdictRecord {
            session_id: row.get("id"),
            question: row.get("question"),
            verdict: row.get("verdict"),
            response_count: row.get::<i64, _>("response_count") as usize,
            participants,
            created_at: row.get::<i64, _>("created_at") as u64,
            finalized_at: row.get::<i64, _>("finalized_at") as u64,
        }
    }
}

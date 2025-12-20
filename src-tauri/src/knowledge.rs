use crate::deliberation::{DeliberationResult, DeliberationRound, MemberResponse};
use crate::logger::{LogLevel, Logger};
use crate::protocol::{CouncilSession, CouncilResponse, SessionStatus};
use crate::reputation::{AgentReputation, AgentTier, ReputationScore};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePool, Row};
use std::sync::Arc;

/// Vector embedding for semantic search
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Embedding {
    pub vector: Vec<f32>,
    pub dimension: usize,
}

/// Chunk of text with embedding for RAG
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct TextChunk {
    pub id: String,
    pub deliberation_id: String,
    pub text: String,
    pub chunk_type: ChunkType,
    pub embedding: Option<Embedding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum ChunkType {
    Question,
    Response { round: usize, member: String },
    Consensus,
}

/// Search result with similarity score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub deliberation_id: String,
    pub question: String,
    pub relevance_score: f32,
    pub text_snippet: String,
}

/// RAG context for deliberation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGContext {
    pub relevant_decisions: Vec<SearchResult>,
    pub context_text: String,
}

/// Knowledge Bank with RAG capabilities
pub struct KnowledgeBank {
    pool: SqlitePool,
    logger: Arc<Logger>,
    ollama_url: String,
    ollama_auth: Option<(String, String)>,
    embedding_model: String,
}

impl KnowledgeBank {
    /// Initialize knowledge bank with database
    pub async fn new(
        db_path: &str,
        logger: Arc<Logger>,
        ollama_url: String,
        ollama_auth: Option<(String, String)>,
    ) -> Result<Self, String> {
        logger.log(
            LogLevel::Info,
            "knowledge",
            &format!("🧠 Initializing Knowledge Bank at {}", db_path),
        );

        let pool = SqlitePool::connect(db_path)
            .await
            .map_err(|e| format!("Failed to connect to database: {}", e))?;

        let kb = Self {
            pool,
            logger: logger.clone(),
            ollama_url,
            ollama_auth,
            embedding_model: "nomic-embed-text".to_string(),
        };

        kb.initialize_schema().await?;
        logger.log(LogLevel::Success, "knowledge", "✅ Knowledge Bank ready");

        Ok(kb)
    }

    /// Create database schema with vector support
    async fn initialize_schema(&self) -> Result<(), String> {
        self.logger
            .log(LogLevel::Debug, "knowledge", "📊 Creating database schema");

        // Main deliberations table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS deliberations (
                id TEXT PRIMARY KEY,
                question TEXT NOT NULL,
                consensus TEXT,
                created_at INTEGER NOT NULL,
                completed BOOLEAN NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create deliberations table: {}", e))?;

        // Topics history table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS topics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                topic TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                created_by TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create topics table: {}", e))?;

        // Rounds table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS rounds (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                deliberation_id TEXT NOT NULL,
                round_number INTEGER NOT NULL,
                FOREIGN KEY (deliberation_id) REFERENCES deliberations(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create rounds table: {}", e))?;

        // Responses table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS responses (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                round_id INTEGER NOT NULL,
                member_name TEXT NOT NULL,
                model TEXT NOT NULL,
                response TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                FOREIGN KEY (round_id) REFERENCES rounds(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create responses table: {}", e))?;

        // Text chunks with embeddings for RAG
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS text_chunks (
                id TEXT PRIMARY KEY,
                deliberation_id TEXT NOT NULL,
                text TEXT NOT NULL,
                chunk_type TEXT NOT NULL,
                chunk_metadata TEXT,
                FOREIGN KEY (deliberation_id) REFERENCES deliberations(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create text_chunks table: {}", e))?;

        // Embeddings table (separate for flexibility)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS embeddings (
                chunk_id TEXT PRIMARY KEY,
                embedding BLOB NOT NULL,
                dimension INTEGER NOT NULL,
                FOREIGN KEY (chunk_id) REFERENCES text_chunks(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create embeddings table: {}", e))?;

        // Reputation table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS reputation (
                agent_id TEXT PRIMARY KEY,
                tier TEXT NOT NULL,
                accuracy REAL NOT NULL,
                reasoning REAL NOT NULL,
                contribution REAL NOT NULL,
                total_votes INTEGER NOT NULL,
                successful_consensus INTEGER NOT NULL,
                last_updated INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create reputation table: {}", e))?;

        // Chat logs table for per-channel RAG/Context
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS chat_logs (
                id TEXT PRIMARY KEY,
                channel TEXT NOT NULL,
                author TEXT NOT NULL,
                author_type TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                signature TEXT,
                reply_to TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create chat_logs table: {}", e))?;

        // Chat embeddings table (future use for semantic search)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS chat_embeddings (
                message_id TEXT PRIMARY KEY,
                embedding BLOB NOT NULL,
                dimension INTEGER NOT NULL,
                FOREIGN KEY (message_id) REFERENCES chat_logs(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create chat_embeddings table: {}", e))?;

        self.logger
            .log(LogLevel::Success, "knowledge", "✅ Database schema initialized");

        Ok(())
    }

    /// Store deliberation result with full RAG processing
    pub async fn store_deliberation(&self, result: &DeliberationResult) -> Result<(), String> {
        self.logger.log(
            LogLevel::Info,
            "knowledge",
            &format!("💾 Storing deliberation: {}", result.session_id),
        );

        // Store main deliberation
        sqlx::query(
            r#"
            INSERT INTO deliberations (id, question, consensus, created_at, completed)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&result.session_id)
        .bind(&result.question)
        .bind(&result.consensus)
        .bind(result.created_at as i64)
        .bind(result.completed)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to store deliberation: {}", e))?;

        // Store rounds and responses
        for round in &result.rounds {
            let round_id = self.store_round(&result.session_id, round).await?;

            for response in &round.responses {
                self.store_response(round_id, response).await?;
            }
        }

        // Generate and store embeddings for RAG
        self.generate_embeddings(result).await?;

        self.logger.log(
            LogLevel::Success,
            "knowledge",
            &format!(
                "✅ Stored deliberation with embeddings: {}",
                result.session_id
            ),
        );

        Ok(())
    }

    /// Store a deliberation round
    async fn store_round(
        &self,
        deliberation_id: &str,
        round: &DeliberationRound,
    ) -> Result<i64, String> {
        let result = sqlx::query(
            r#"
            INSERT INTO rounds (deliberation_id, round_number)
            VALUES (?, ?)
            "#,
        )
        .bind(deliberation_id)
        .bind(round.round_number as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to store round: {}", e))?;

        Ok(result.last_insert_rowid())
    }

    /// Store a response
    async fn store_response(&self, round_id: i64, response: &MemberResponse) -> Result<(), String> {
        sqlx::query(
            r#"
            INSERT INTO responses (round_id, member_name, model, response, timestamp)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(round_id)
        .bind(&response.member_name)
        .bind(&response.model)
        .bind(&response.response)
        .bind(response.timestamp as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to store response: {}", e))?;

        Ok(())
    }

    /// Generate embeddings for all chunks of a deliberation
    async fn generate_embeddings(&self, result: &DeliberationResult) -> Result<(), String> {
        self.logger.log(
            LogLevel::Debug,
            "knowledge",
            &format!(
                "🔢 Generating embeddings for deliberation: {}",
                result.session_id
            ),
        );

        let mut chunks = Vec::new();

        // Chunk 1: The question
        chunks.push((
            format!("{}-question", result.session_id),
            result.question.clone(),
            "question".to_string(),
        ));

        // Chunk 2-N: Each response
        for (round_idx, round) in result.rounds.iter().enumerate() {
            for (resp_idx, response) in round.responses.iter().enumerate() {
                let chunk_id = format!("{}-r{}-resp{}", result.session_id, round_idx, resp_idx);
                let metadata = serde_json::json!({
                    "round": round.round_number,
                    "member": response.member_name
                })
                .to_string();
                chunks.push((chunk_id, response.response.clone(), metadata));
            }
        }

        // Chunk N+1: Consensus (if exists)
        if let Some(consensus) = &result.consensus {
            chunks.push((
                format!("{}-consensus", result.session_id),
                consensus.clone(),
                "consensus".to_string(),
            ));
        }

        // Store chunks and generate embeddings in parallel
        let total_chunks = chunks.len();
        for (chunk_id, text, metadata) in &chunks {
            // Store text chunk
            sqlx::query(
                r#"
                INSERT INTO text_chunks (id, deliberation_id, text, chunk_type, chunk_metadata)
                VALUES (?, ?, ?, ?, ?)
                "#,
            )
            .bind(chunk_id)
            .bind(&result.session_id)
            .bind(text)
            .bind("text")
            .bind(metadata)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to store chunk: {}", e))?;

            // Generate embedding via Ollama
            let embedding = self.generate_embedding(text).await?;

            // Store embedding as BLOB
            let embedding_bytes = Self::serialize_embedding(&embedding);
            sqlx::query(
                r#"
                INSERT INTO embeddings (chunk_id, embedding, dimension)
                VALUES (?, ?, ?)
                "#,
            )
            .bind(chunk_id)
            .bind(&embedding_bytes)
            .bind(embedding.len() as i64)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to store embedding: {}", e))?;
        }

        self.logger.log(
            LogLevel::Success,
            "knowledge",
            &format!("✅ Generated {} embeddings", total_chunks),
        );

        Ok(())
    }

    /// Generate embedding using Ollama
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, String> {
        // Use the existing Ollama client
        // Note: This assumes the embedding model is pulled and available
        let client = reqwest::Client::new();
        let url = format!("{}/api/embeddings", self.ollama_url);
        
        let payload = serde_json::json!({
            "model": self.embedding_model,
            "prompt": text
        });

        let mut request = client.post(&url).json(&payload);
        
        // Add basic auth if configured
        if let Some((username, password)) = &self.ollama_auth {
            request = request.basic_auth(username, Some(password));
        }

        let res = request
            .send()
            .await
            .map_err(|e| format!("Failed to call Ollama: {}", e))?;

        if !res.status().is_success() {
            return Err(format!("Ollama error: {}", res.status()));
        }

        let body: serde_json::Value = res
            .json()
            .await
            .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

        let embedding = body["embedding"]
            .as_array()
            .ok_or("Invalid embedding format")?
            .iter()
            .map(|v| v.as_f64().unwrap() as f32)
            .collect();

        Ok(embedding)
    }

    /// Store embedding in DB
    async fn store_embedding(&self, chunk_id: &str, embedding: &[f32]) -> Result<(), String> {
        let embedding_bytes = Self::serialize_embedding(embedding);
        
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO embeddings (chunk_id, embedding, dimension)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(chunk_id)
        .bind(embedding_bytes)
        .bind(embedding.len() as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to store embedding: {}", e))?;

        Ok(())
    }

    fn serialize_embedding(embedding: &[f32]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(embedding.len() * 4);
        for val in embedding {
            bytes.extend_from_slice(&val.to_le_bytes());
        }
        bytes
    }

    fn deserialize_embedding(bytes: &[u8]) -> Vec<f32> {
        bytes
            .chunks(4)
            .map(|chunk| {
                let mut buf = [0u8; 4];
                buf.copy_from_slice(chunk);
                f32::from_le_bytes(buf)
            })
            .collect()
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }

    fn truncate(s: &str, max_chars: usize) -> String {
        if s.len() <= max_chars {
            s.to_string()
        } else {
            format!("{}...", &s[..max_chars])
        }
    }

    /// Search for relevant deliberations (Semantic Search)
    pub async fn semantic_search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, String> {
        self.logger.log(
            LogLevel::Debug,
            "knowledge",
            &format!("🔍 Searching for: {}", query),
        );

        let query_embedding = self.generate_embedding(query).await?;
        
        // Fetch all embeddings (naive approach for MVP)
        // In production, use vector extension or specialized DB
        let rows = sqlx::query(
            r#"
            SELECT e.embedding, t.deliberation_id, t.text, d.question
            FROM embeddings e
            JOIN text_chunks t ON e.chunk_id = t.id
            JOIN deliberations d ON t.deliberation_id = d.id
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch embeddings: {}", e))?;

        let mut results = Vec::new();

        for row in rows {
            let delib_id: String = row.get("deliberation_id");
            let question: String = row.get("question");
            let text: String = row.get("text");
            let embedding_bytes: Vec<u8> = row.get("embedding");

            let embedding = Self::deserialize_embedding(&embedding_bytes);
            let similarity = Self::cosine_similarity(&query_embedding, &embedding);

            results.push(SearchResult {
                deliberation_id: delib_id,
                question,
                relevance_score: similarity,
                text_snippet: Self::truncate(&text, 200),
            });
        }

        // Sort by similarity and take top results
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        results.truncate(limit);

        self.logger.log(
            LogLevel::Success,
            "knowledge",
            &format!("✅ Found {} relevant results", results.len()),
        );

        Ok(results)
    }

    /// Build RAG context string from query
    pub async fn build_rag_context(&self, query: &str, limit: usize) -> Result<RAGContext, String> {
        let results = self.semantic_search(query, limit).await?;
        
        let mut context_text = String::from("### Relevant Past Decisions:\n\n");
        for result in &results {
            context_text.push_str(&format!("- **Question:** {}\n  **Snippet:** {}\n\n", 
                result.question, result.text_snippet));
        }
        
        Ok(RAGContext {
            relevant_decisions: results,
            context_text,
        })
    }

    /// Get a specific deliberation by ID
    pub async fn get_deliberation(
        &self,
        deliberation_id: &str,
    ) -> Result<DeliberationResult, String> {
        self.logger.log(
            LogLevel::Info,
            "knowledge",
            &format!("📖 Retrieving deliberation: {}", deliberation_id),
        );

        // Get main deliberation
        let row = sqlx::query(
            r#"
            SELECT id, question, consensus, created_at, completed
            FROM deliberations
            WHERE id = ?
            "#,
        )
        .bind(deliberation_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Deliberation not found: {}", e))?;

        let session_id: String = row.get("id");
        let question: String = row.get("question");
        let consensus: Option<String> = row.get("consensus");
        let created_at: i64 = row.get("created_at");
        let completed: bool = row.get("completed");

        // Get rounds
        let round_rows = sqlx::query(
            r#"
            SELECT id, round_number
            FROM rounds
            WHERE deliberation_id = ?
            ORDER BY round_number
            "#,
        )
        .bind(&session_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch rounds: {}", e))?;

        let mut rounds = Vec::new();
        for round_row in round_rows {
            let round_id: i64 = round_row.get("id");
            let round_number: i64 = round_row.get("round_number");

            // Get responses for this round
            let response_rows = sqlx::query(
                r#"
                SELECT member_name, model, response, timestamp
                FROM responses
                WHERE round_id = ?
                "#,
            )
            .bind(round_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Failed to fetch responses: {}", e))?;

            let mut responses = Vec::new();
            for resp_row in response_rows {
                let member_name: String = resp_row.get("member_name");
                let model: String = resp_row.get("model");
                let response: String = resp_row.get("response");
                let timestamp: i64 = resp_row.get("timestamp");

                responses.push(crate::deliberation::MemberResponse {
                    member_name,
                    model,
                    response,
                    timestamp: timestamp as u64,
                });
            }

            rounds.push(DeliberationRound {
                round_number: round_number as usize,
                responses,
            });
        }

        Ok(DeliberationResult {
            session_id,
            question,
            consensus,
            rounds,
            created_at: created_at as u64,
            completed,
        })
    }

    /// Get all deliberations (for debugging)
    pub async fn list_all(&self) -> Result<Vec<(String, String, bool)>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, question, completed FROM deliberations ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to list deliberations: {}", e))?;

        Ok(rows
            .iter()
            .map(|row| (row.get("id"), row.get("question"), row.get("completed")))
            .collect())
    }

    /// Record a new topic in history
    pub async fn add_topic(&self, topic: &str, created_by: Option<&str>) -> Result<(), String> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs() as i64;

        sqlx::query(
            r#"
            INSERT INTO topics (topic, created_at, created_by)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(topic)
        .bind(timestamp)
        .bind(created_by)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to record topic: {}", e))?;

        self.logger.log(
            LogLevel::Debug,
            "knowledge",
            &format!("📚 Recorded topic in history: {}", topic),
        );

        Ok(())
    }

    /// Get recent topics
    pub async fn get_recent_topics(&self, limit: i64) -> Result<Vec<(String, i64)>, String> {
        let rows = sqlx::query(
            r#"
            SELECT topic, created_at FROM topics
            ORDER BY created_at DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch topics: {}", e))?;

        let topics = rows
            .into_iter()
            .map(|row| {
                (
                    row.get::<String, _>("topic"),
                    row.get::<i64, _>("created_at"),
                )
            })
            .collect();

        Ok(topics)
    }

    /// Save agent reputation
    pub async fn save_reputation(&self, reputation: &AgentReputation) -> Result<(), String> {
        let tier_str = serde_json::to_string(&reputation.tier).unwrap_or_default();
        
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO reputation 
            (agent_id, tier, accuracy, reasoning, contribution, total_votes, successful_consensus, last_updated)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&reputation.agent_id)
        .bind(tier_str)
        .bind(reputation.score.accuracy)
        .bind(reputation.score.reasoning)
        .bind(reputation.score.contribution)
        .bind(reputation.score.total_votes)
        .bind(reputation.score.successful_consensus)
        .bind(reputation.last_updated as i64)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to save reputation: {}", e))?;

        Ok(())
    }

    /// Load all agent reputations
    pub async fn load_reputations(&self) -> Result<Vec<AgentReputation>, String> {
        let rows = sqlx::query(
            r#"
            SELECT agent_id, tier, accuracy, reasoning, contribution, total_votes, successful_consensus, last_updated
            FROM reputation
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load reputations: {}", e))?;

        let mut reputations = Vec::new();

        for row in rows {
            let tier_str: String = row.get("tier");
            let tier: AgentTier = serde_json::from_str(&tier_str).unwrap_or(AgentTier::Candidate);

            reputations.push(AgentReputation {
                agent_id: row.get("agent_id"),
                tier,
                score: ReputationScore {
                    accuracy: row.get("accuracy"),
                    reasoning: row.get("reasoning"),
                    contribution: row.get("contribution"),
                    total_votes: row.get("total_votes"),
                    successful_consensus: row.get("successful_consensus"),
                },
                last_updated: row.get::<i64, _>("last_updated") as u64,
            });
        }

        Ok(reputations)
    }

    /// Save or update council session
    pub async fn save_session(&self, session: &CouncilSession) -> Result<(), String> {
        // Save session
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO deliberations (id, question, consensus, created_at, completed)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&session.id)
        .bind(&session.question)
        .bind(&session.consensus)
        .bind(session.created_at as i64)
        .bind(session.status == SessionStatus::ConsensusReached)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to save session: {}", e))?;

        // Save responses (we delete all and re-insert for simplicity, or check existence)
        // For now, let's just insert new ones. 
        // Actually, simpler to just delete all for this session and re-insert.
        // This is inefficient but safe for MVP.
        
        sqlx::query("DELETE FROM responses WHERE round_id IN (SELECT id FROM rounds WHERE deliberation_id = ?)")
            .bind(&session.id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to clear old responses: {}", e))?;
            
        sqlx::query("DELETE FROM rounds WHERE deliberation_id = ?")
            .bind(&session.id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to clear old rounds: {}", e))?;

        // Create a single "round" for now as our current model is single-round
        let round_id = sqlx::query(
            r#"
            INSERT INTO rounds (deliberation_id, round_number)
            VALUES (?, 1)
            RETURNING id
            "#,
        )
        .bind(&session.id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to create round: {}", e))?
        .get::<i64, _>("id");

        for response in &session.responses {
            sqlx::query(
                r#"
                INSERT INTO responses (round_id, member_name, model, response, timestamp)
                VALUES (?, ?, ?, ?, ?)
                "#,
            )
            .bind(round_id)
            .bind(&response.peer_id) // Using peer_id as member_name
            .bind(&response.model_name)
            .bind(&response.response)
            .bind(response.timestamp as i64)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to save response: {}", e))?;
        }

        Ok(())
    }

    /// Load all council sessions
    pub async fn load_sessions(&self) -> Result<Vec<CouncilSession>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, question, consensus, created_at, completed
            FROM deliberations
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to load sessions: {}", e))?;

        let mut sessions = Vec::new();

        for row in rows {
            let id: String = row.get("id");
            let completed: bool = row.get("completed");
            
            // Load responses
            let response_rows = sqlx::query(
                r#"
                SELECT r.member_name, r.model, r.response, r.timestamp
                FROM responses r
                JOIN rounds ro ON r.round_id = ro.id
                WHERE ro.deliberation_id = ?
                "#,
            )
            .bind(&id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Failed to load responses: {}", e))?;

            let mut responses = Vec::new();
            for r_row in response_rows {
                responses.push(CouncilResponse {
                    model_name: r_row.get("model"),
                    response: r_row.get("response"),
                    peer_id: r_row.get("member_name"),
                    timestamp: r_row.get::<i64, _>("timestamp") as u64,
                    signature: None, // Not stored in DB yet
                    public_key: None,
                });
            }

            sessions.push(CouncilSession {
                id,
                question: row.get("question"),
                responses,
                commitments: Vec::new(), // Not persisted
                reveals: Vec::new(),     // Not persisted
                consensus: row.get("consensus"),
                status: if completed { SessionStatus::ConsensusReached } else { SessionStatus::GatheringResponses },
                created_at: row.get::<i64, _>("created_at") as u64,
            });
        }

        Ok(sessions)
    }

    /// Save a chat message to the knowledge bank
    pub async fn save_chat_message(&self, message: &crate::chat::Message) -> Result<(), String> {
        sqlx::query(
            r#"
            INSERT INTO chat_logs (id, channel, author, author_type, content, timestamp, signature, reply_to)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&message.id)
        .bind(message.channel.as_str())
        .bind(&message.author)
        .bind(format!("{:?}", message.author_type)) // Serialize enum as string
        .bind(&message.content)
        .bind(message.timestamp.timestamp())
        .bind(&message.signature)
        .bind(&message.reply_to)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to save chat message: {}", e))?;

        // Generate embedding for RAG (per-channel context)
        // Only for non-system messages to save resources
        if message.author_type != crate::chat::AuthorType::System {
            if let Ok(embedding) = self.generate_embedding(&message.content).await {
                // Store in chat_embeddings table
                let embedding_bytes = Self::serialize_embedding(&embedding);
                sqlx::query(
                    r#"
                    INSERT OR REPLACE INTO chat_embeddings (message_id, embedding, dimension)
                    VALUES (?, ?, ?)
                    "#,
                )
                .bind(&message.id)
                .bind(&embedding_bytes)
                .bind(embedding.len() as i64)
                .execute(&self.pool)
                .await
                .map_err(|e| format!("Failed to store chat embedding: {}", e))?;
            }
        }

        Ok(())
    }

    /// Get recent chat messages for a channel (Context)
    pub async fn get_chat_history(
        &self,
        channel: crate::chat::ChannelType,
        limit: usize,
    ) -> Result<Vec<crate::chat::Message>, String> {
        let rows = sqlx::query(
            r#"
            SELECT id, channel, author, author_type, content, timestamp, signature, reply_to
            FROM chat_logs
            WHERE channel = ?
            ORDER BY timestamp DESC
            LIMIT ?
            "#,
        )
        .bind(channel.as_str())
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch chat history: {}", e))?;

        let mut messages = Vec::new();
        for row in rows {
            let channel_str: String = row.get("channel");
            let author_type_str: String = row.get("author_type");
            let timestamp_int: i64 = row.get("timestamp");

            let channel_type = crate::chat::ChannelType::from_str(&channel_str)
                .ok_or_else(|| format!("Invalid channel type: {}", channel_str))?;

            let author_type = match author_type_str.as_str() {
                "Human" => crate::chat::AuthorType::Human,
                "AI" => crate::chat::AuthorType::AI,
                "System" => crate::chat::AuthorType::System,
                _ => crate::chat::AuthorType::Human, // Default fallback
            };

            let timestamp = chrono::DateTime::<chrono::Utc>::from_timestamp(timestamp_int, 0)
                .ok_or("Invalid timestamp")?;

            messages.push(crate::chat::Message {
                id: row.get("id"),
                channel: channel_type,
                author: row.get("author"),
                author_type,
                content: row.get("content"),
                timestamp,
                signature: row.get("signature"),
                reply_to: row.get("reply_to"),
                reactions: Vec::new(), // Reactions not persisted yet
            });
        }

        // Return in chronological order (oldest first)
        messages.reverse();
        Ok(messages)
    }

    /// Add a text chunk manually (e.g. for consensus results)
    pub async fn add_text_chunk(
        &self,
        id: &str,
        deliberation_id: &str,
        text: &str,
        chunk_type: ChunkType,
    ) -> Result<(), String> {
        // Serialize chunk type
        let chunk_type_str = match chunk_type {
            ChunkType::Question => "Question",
            ChunkType::Response { .. } => "Response",
            ChunkType::Consensus => "Consensus",
        };

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO text_chunks (id, deliberation_id, text, chunk_type)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(deliberation_id)
        .bind(text)
        .bind(chunk_type_str)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to add text chunk: {}", e))?;

        // Generate embedding (async/background ideally, but inline for now)
        if let Ok(embedding) = self.generate_embedding(text).await {
            self.store_embedding(id, &embedding).await?;
        }

        Ok(())
    }

    /// Search chat history for a specific channel (Scoped RAG)
    pub async fn search_channel_context(
        &self,
        channel: crate::chat::ChannelType,
        query: &str,
        limit: usize,
    ) -> Result<Vec<String>, String> {
        let query_embedding = self.generate_embedding(query).await?;
        
        // Join chat_logs and chat_embeddings, filter by channel
        let rows = sqlx::query(
            r#"
            SELECT c.content, e.embedding
            FROM chat_logs c
            JOIN chat_embeddings e ON c.id = e.message_id
            WHERE c.channel = ?
            "#,
        )
        .bind(channel.as_str())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch channel embeddings: {}", e))?;

        let mut results = Vec::new();

        for row in rows {
            let content: String = row.get("content");
            let embedding_bytes: Vec<u8> = row.get("embedding");
            let embedding = Self::deserialize_embedding(&embedding_bytes);
            
            let similarity = Self::cosine_similarity(&query_embedding, &embedding);
            results.push((content, similarity));
        }

        // Sort by similarity
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results.truncate(limit);

        Ok(results.into_iter().map(|(content, _)| content).collect())
    }

    /// Clear chat context for a channel (Reset Period)
    pub async fn clear_channel_context(&self, channel: crate::chat::ChannelType) -> Result<(), String> {
        // Delete embeddings first (foreign key)
        sqlx::query(
            r#"
            DELETE FROM chat_embeddings 
            WHERE message_id IN (SELECT id FROM chat_logs WHERE channel = ?)
            "#,
        )
        .bind(channel.as_str())
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to clear chat embeddings: {}", e))?;
        
        self.logger.log(
            LogLevel::Info,
            "knowledge",
            &format!("🧹 Cleared RAG context for #{}", channel.as_str()),
        );

        Ok(())
    }

    // Removed duplicate store_embedding
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_knowledge_bank_initialization() {
        let logger = Arc::new(Logger::new(false));
        let kb = KnowledgeBank::new(
            "sqlite::memory:",
            logger,
            "http://localhost:11434".to_string(),
            None, // No auth for tests
        )
        .await;

        assert!(kb.is_ok());
    }

    #[tokio::test]
    async fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let c = vec![0.0, 1.0, 0.0];

        assert!((KnowledgeBank::cosine_similarity(&a, &b) - 1.0).abs() < 0.001);
        assert!((KnowledgeBank::cosine_similarity(&a, &c) - 0.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_embedding_serialization() {
        let embedding = vec![1.5, -2.3, 0.0, 42.1];
        let bytes = KnowledgeBank::serialize_embedding(&embedding);
        let deserialized = KnowledgeBank::deserialize_embedding(&bytes);

        assert_eq!(embedding.len(), deserialized.len());
        for (a, b) in embedding.iter().zip(deserialized.iter()) {
            assert!((a - b).abs() < 0.001);
        }
    }
}

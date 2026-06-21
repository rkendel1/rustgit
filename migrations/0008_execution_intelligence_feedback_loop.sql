CREATE TABLE IF NOT EXISTS execution_embeddings (
    id TEXT PRIMARY KEY,
    repository_id TEXT NOT NULL REFERENCES repositories(repo_id) ON DELETE CASCADE,
    commit_sha TEXT NOT NULL,
    fingerprint_hash TEXT NOT NULL,
    embedding VECTOR(8) NOT NULL,
    language TEXT NOT NULL,
    framework TEXT NOT NULL,
    runtime TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS execution_patterns (
    id TEXT PRIMARY KEY,
    fingerprint TEXT NOT NULL,
    failure_type TEXT NOT NULL,
    repair TEXT NOT NULL,
    success_rate DOUBLE PRECISION NOT NULL CHECK (success_rate >= 0 AND success_rate <= 1),
    execution_count BIGINT NOT NULL DEFAULT 0,
    average_duration DOUBLE PRECISION NOT NULL DEFAULT 0,
    average_cost DOUBLE PRECISION NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS execution_contexts (
    execution_id TEXT PRIMARY KEY REFERENCES executions(execution_id) ON DELETE CASCADE,
    similar_execution_ids TEXT[] NOT NULL DEFAULT '{}',
    retrieved_patterns TEXT[] NOT NULL DEFAULT '{}',
    generated_plan TEXT NOT NULL,
    chosen_plan TEXT NOT NULL
);

CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE IF NOT EXISTS repository_context_snapshots (
    snapshot_id TEXT PRIMARY KEY,
    repository_id TEXT NOT NULL REFERENCES repositories(repo_id) ON DELETE CASCADE,
    context_payload JSONB NOT NULL,
    captured_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS repository_questions (
    question_id TEXT PRIMARY KEY,
    repository_id TEXT NOT NULL REFERENCES repositories(repo_id) ON DELETE CASCADE,
    question TEXT NOT NULL,
    context_snapshot_id TEXT REFERENCES repository_context_snapshots(snapshot_id) ON DELETE SET NULL,
    asked_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS repository_answers (
    answer_id TEXT PRIMARY KEY,
    question_id TEXT NOT NULL REFERENCES repository_questions(question_id) ON DELETE CASCADE,
    answer TEXT NOT NULL,
    confidence DOUBLE PRECISION NOT NULL CHECK (confidence >= 0 AND confidence <= 1),
    outcome TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS repository_embeddings (
    embedding_id TEXT PRIMARY KEY,
    repository_id TEXT NOT NULL REFERENCES repositories(repo_id) ON DELETE CASCADE,
    artifact_kind TEXT NOT NULL,
    artifact_id TEXT NOT NULL,
    content TEXT NOT NULL,
    embedding VECTOR(1536) NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

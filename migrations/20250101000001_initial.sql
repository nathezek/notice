-- =============================================
-- Notice V2 — Initial Schema
-- =============================================

-- ─────────────────────────────────────────────
-- Extensions
-- ─────────────────────────────────────────────
CREATE EXTENSION IF NOT EXISTS "pgcrypto";  -- for gen_random_uuid()

-- ─────────────────────────────────────────────
-- Documents (scraped web content)
-- ─────────────────────────────────────────────
-- This is the primary content table. Every scraped
-- web page becomes a row here. MeiliBridge syncs
-- this table to Meilisearch for full-text search.

CREATE TABLE documents (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    url         TEXT NOT NULL UNIQUE,
    domain      TEXT NOT NULL,
    title       TEXT,
    raw_content TEXT NOT NULL,
    summary     TEXT,
    status      TEXT NOT NULL DEFAULT 'pending_summary'
                CHECK (status IN ('pending_summary', 'summarized', 'failed')),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_documents_domain     ON documents (domain);
CREATE INDEX idx_documents_status     ON documents (status);
CREATE INDEX idx_documents_created_at ON documents (created_at DESC);

-- ─────────────────────────────────────────────
-- Users
-- ─────────────────────────────────────────────
CREATE TABLE users (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username      TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ─────────────────────────────────────────────
-- Knowledge Graph — Entities (nodes)
-- ─────────────────────────────────────────────
-- Each row is a concept/topic/entity that a
-- specific user has shown interest in. The weight
-- increases every time the entity is reinforced
-- through search behavior.

CREATE TABLE kg_entities (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    entity_type TEXT NOT NULL DEFAULT 'topic',
    weight      DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (user_id, name, entity_type)
);

CREATE INDEX idx_kg_entities_user_weight ON kg_entities (user_id, weight DESC);

-- ─────────────────────────────────────────────
-- Knowledge Graph — Relationships (edges)
-- ─────────────────────────────────────────────
CREATE TABLE kg_relationships (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id           UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    from_entity_id    UUID NOT NULL REFERENCES kg_entities (id) ON DELETE CASCADE,
    to_entity_id      UUID NOT NULL REFERENCES kg_entities (id) ON DELETE CASCADE,
    relationship_type TEXT NOT NULL DEFAULT 'related_to',
    weight            DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (user_id, from_entity_id, to_entity_id, relationship_type)
);

CREATE INDEX idx_kg_rels_from ON kg_relationships (from_entity_id);
CREATE INDEX idx_kg_rels_to   ON kg_relationships (to_entity_id);

-- ─────────────────────────────────────────────
-- Search History
-- ─────────────────────────────────────────────
-- Records every search for:
-- 1. Session context (disambiguation)
-- 2. Implicit KG building (extract entities from queries)

CREATE TABLE search_history (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id       UUID REFERENCES users (id) ON DELETE SET NULL,
    session_id    TEXT,
    query         TEXT NOT NULL,
    intent        TEXT NOT NULL DEFAULT 'search',
    results_count INTEGER NOT NULL DEFAULT 0,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_search_history_user    ON search_history (user_id, created_at DESC);
CREATE INDEX idx_search_history_session ON search_history (session_id, created_at DESC);

-- ─────────────────────────────────────────────
-- Crawl Queue
-- ─────────────────────────────────────────────
-- URLs waiting to be scraped by the background
-- crawler. Uses SELECT ... FOR UPDATE SKIP LOCKED
-- for safe concurrent dequeuing.

CREATE TABLE crawl_queue (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    url          TEXT NOT NULL UNIQUE,
    status       TEXT NOT NULL DEFAULT 'pending'
                 CHECK (status IN ('pending', 'in_progress', 'completed', 'failed')),
    priority     INTEGER NOT NULL DEFAULT 0,
    retry_count  INTEGER NOT NULL DEFAULT 0,
    max_retries  INTEGER NOT NULL DEFAULT 3,
    last_error   TEXT,
    submitted_by UUID REFERENCES users (id) ON DELETE SET NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_crawl_queue_dequeue ON crawl_queue (status, priority DESC, created_at ASC);

-- ─────────────────────────────────────────────
-- Auto-update updated_at columns
-- ─────────────────────────────────────────────
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_documents_updated_at
    BEFORE UPDATE ON documents
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_kg_entities_updated_at
    BEFORE UPDATE ON kg_entities
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER trg_crawl_queue_updated_at
    BEFORE UPDATE ON crawl_queue
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- ─────────────────────────────────────────────
-- MeiliBridge Publication (CDC)
-- ─────────────────────────────────────────────
-- Only the documents table is synced to Meilisearch.
-- MeiliBridge uses logical replication to capture
-- INSERT/UPDATE/DELETE events from this publication.

CREATE PUBLICATION meilibridge_pub FOR TABLE documents;

-- Migration: Create query_summaries table for AI answer caching
CREATE TABLE IF NOT EXISTS query_summaries (
    query TEXT PRIMARY KEY,
    answer JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Index for faster lookup (though query is already PK)
CREATE INDEX IF NOT EXISTS idx_query_summaries_query ON query_summaries(query);

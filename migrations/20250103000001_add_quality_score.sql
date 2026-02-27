-- Add quality_score to documents table
ALTER TABLE documents ADD COLUMN quality_score DOUBLE PRECISION NOT NULL DEFAULT 1.0;

-- Index for ranking
CREATE INDEX idx_documents_quality_score ON documents (quality_score DESC);

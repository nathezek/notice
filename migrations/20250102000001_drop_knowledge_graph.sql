-- =============================================
-- Drop Knowledge Graph tables
-- =============================================
-- The KG feature injected noise into search results
-- by expanding queries with unrelated terms from the
-- user's search history. Removing it entirely.

DROP TABLE IF EXISTS kg_relationships;
DROP TABLE IF EXISTS kg_entities;

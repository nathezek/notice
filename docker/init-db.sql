-- =============================================
-- MeiliBridge replication user
-- =============================================
-- This user is used by MeiliBridge to stream
-- changes from PostgreSQL to Meilisearch via
-- logical replication (CDC).

CREATE USER meilibridge WITH REPLICATION LOGIN PASSWORD 'meilibridge_pass';

-- Allow connecting to the notice database
GRANT CONNECT ON DATABASE notice TO meilibridge;

-- Allow reading from public schema
GRANT USAGE ON SCHEMA public TO meilibridge;

-- Grant SELECT on all current tables (none yet, but safe to run)
GRANT SELECT ON ALL TABLES IN SCHEMA public TO meilibridge;

-- Automatically grant SELECT on any future tables
ALTER DEFAULT PRIVILEGES IN SCHEMA public
    GRANT SELECT ON TABLES TO meilibridge;

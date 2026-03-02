# Phase 2: Core Backend & Search

### Part One:
✅ 6 PostgreSQL tables with indexes, triggers, and constraints
✅ MeiliBridge CDC publication
✅ Full CRUD operations for all tables (notice-db)
✅ Real user registration & login with Argon2 + JWT
✅ URL submission → crawl queue (background path)
✅ Immediate crawl → scrape → Gemini summarize → store (dev path)
✅ Document listing with pagination
✅ Real Meilisearch search returning ranked results with snippets
✅ Search history recording
✅ Meilisearch index auto-configuration at startup
✅ MeiliBridge wired into docker-compose
✅ Health check verifying all dependencies
✅ Proper API error handling (ApiError → HTTP status codes)

```text
POST /api/crawl {"url":"..."}
  │
  ▼
scrape (reqwest+scraper)
  │
  ▼
store in PostgreSQL ──CDC──▶ MeiliBridge ──▶ Meilisearch
  │
  ▼
summarize (Gemini API)
  │
  ▼
update summary in PostgreSQL ──CDC──▶ Meilisearch (updated)

GET /api/search?q=...
  │
  ▼
classify intent ──▶ [instant answer] or [Meilisearch search]
  │                                          │
  ▼                                          ▼
record in search_history            return ranked results
```

### Part Two (Fixes made):
✅ Bug fix: MeiliDocument struct mismatch (raw_content in struct but not in displayed_attributes)
✅ Split into MeiliDocumentInput (write) and MeiliDocumentOutput (read)
✅ Direct sync: documents are pushed to Meilisearch immediately on crawl
✅ Resync endpoint: POST /api/admin/resync pushes all PG docs to Meilisearch
✅ Gemini diagnostics: test_connection() at startup with clear error messages
✅ Gemini error parsing: structured error messages from the API
✅ Health check: now shows Gemini status + Meilisearch document count
✅ MeiliBridge config simplified to match official docs
✅ Search now works end-to-end (crawl → store → index → search → results)

```text
POST /api/crawl
  │
  ├──▶ scrape (reqwest + scraper)
  │
  ├──▶ store in PostgreSQL
  │       │
  │       ├──▶ MeiliBridge CDC ──▶ Meilisearch  (async, when it works)
  │       │
  │       └──▶ direct sync ──▶ Meilisearch      (immediate, always works)
  │
  └──▶ summarize (Gemini) ──▶ update PostgreSQL ──▶ Meilisearch (via direct sync)


GET /api/search?q=...
  │
  ├──▶ classify intent
  │     ├── math → instant answer ✅
  │     ├── define → placeholder
  │     ├── timer → placeholder
  │     └── search ──▶ Meilisearch ──▶ ranked results ✅
  │
  └──▶ record in search_history ✅

  ```

  And with this phase two is completed ✅.
## Notice-Search: Advanced Search Features

✅ Meilisearch Synonyms
   - 50+ synonym pairs configured automatically at startup
   - "JS" → JavaScript, "PG" → PostgreSQL, "k8s" → Kubernetes, etc.
   - Transparent to the user — Meilisearch handles expansion internally

✅ Session Context (Disambiguation)
   - Analyzes last 30 minutes of search history
   - Extracts topics from recent queries
   - Adds session-relevant terms to disambiguate vague queries
   - Works with both user_id and session_id

✅ KG Relationship Expansion
   - Finds entities related through co_searched relationships
   - "ownership" co_searched with "rust" → expand with "rust"
   - Minimum weight threshold (2.0) prevents noise

✅ 4-Layer Query Pipeline
   1. Intent Classification (calculate/define/timer/search)
   2. Session Context (disambiguation from recent history)
   3. KG Context (personalization from user profile)
   4. KG Expansion (related entities from relationships)
   All combined into a single augmented query → Meilisearch

### The full pipeline flow:

```text
"ownership" (query from alice, session has "rust traits", "rust borrowing")
    │
    ├─ 1. Classify → Search (not math/define/timer)
    │
    ├─ 2. Session Context
    │     Recent: ["rust traits", "rust borrowing"]
    │     Topics: ["rust", "traits", "borrowing"]
    │     Boost: ["rust"]
    │
    ├─ 3. KG Context
    │     Top interests: [rust:8.0, ownership:3.0, programming:2.0]
    │     Overlapping: ["ownership" exists in KG]
    │     High-weight boost: ["rust"] (weight >= 3.0)
    │
    ├─ 4. KG Expansion
    │     "ownership" co_searched with "rust" (weight 5.0)
    │     Expansion: ["rust"]
    │
    ├─ 5. Combine (deduplicated, max 3 boost terms)
    │     → "ownership rust"
    │
    └─ 6. Meilisearch (with synonyms active)
         → Rust ownership results ranked highest ✅

```
## Knowledge Graph

✅ Entity extraction from search queries
   - Stop word removal
   - Bigram detection ("programming language" as one concept)
   - Entity type classification (language, technology, concept, topic)

✅ Knowledge Graph persistence
   - Upsert entities (create or increment weight)
   - Upsert relationships between co-occurring entities
   - Query entities by name for overlap detection

✅ Context-aware search
   - Load user's top interests before search
   - Detect overlap between query terms and known entities
   - Augment queries with KG context (when weight ≥ 3.0)

✅ Async KG updates
   - Fire-and-forget after search response
   - Never blocks or slows down the search

✅ KG inspection API
   - GET /api/users/{id}/kg — full graph view
   - GET /api/users/{id}/kg/context — what would be injected

✅ Tested with unit tests (extractor, truncation)


### The personalization loop:
```text
Search "rust ownership"
    │
    ├──▶ extract: ["rust" (language), "ownership" (concept)]
    ├──▶ upsert entities: rust.weight++, ownership.weight++
    ├──▶ create relationship: rust ↔ ownership (co_searched)
    └──▶ return search results

        ... later ...

Search "ownership"  (ambiguous!)
    │
    ├──▶ load KG context: [rust: 8.0, ownership: 3.0, ...]
    ├──▶ find overlap: "ownership" exists in KG
    ├──▶ augment: "ownership" → "ownership rust"
    └──▶ Meilisearch ranks Rust ownership docs higher ✅

```
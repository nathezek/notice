✅ Background crawler with concurrent workers (configurable)
✅ Proper User-Agent header (fixes Wikipedia 403 and other sites)
✅ robots.txt checking with in-memory caching
✅ Per-domain rate limiting (1 req/sec default, configurable)
✅ Link discovery (extracts links from crawled pages, enqueues new ones)
✅ Smart URL filtering (skips images, PDFs, login pages, etc.)
✅ Retry with backoff (via crawl_queue retry_count/max_retries)
✅ Stale job recovery on startup (resets stuck in_progress items)
✅ Graceful shutdown (cancellation token)
✅ Crawler status API (pages crawled, failed, links discovered)
✅ Crawler stop API
✅ All environment-configurable (workers, delay, timeout, etc.)


### How the crawler flows:
```text
POST /api/submit {"url":"https://..."}
  │
  └──▶ crawl_queue (status: pending)
         │
         ▼ (background worker picks it up)
  ┌──────────────┐
  │ robots.txt?  │──blocked──▶ mark_failed
  └──────┬───────┘
         │ allowed
         ▼
  ┌──────────────┐
  │ rate limit   │  (wait for domain cooldown)
  └──────┬───────┘
         │
         ▼
  ┌──────────────┐
  │ scrape page  │──error──▶ mark_failed (retry if under max_retries)
  └──────┬───────┘
         │
         ├──▶ extract links ──▶ enqueue new URLs (priority: -1)
         │
         ▼
  ┌──────────────┐
  │ store in PG  │
  └──────┬───────┘
         │
         ▼
  ┌──────────────┐
  │ Gemini       │  (summarize)
  └──────┬───────┘
         │
         ▼
  ┌──────────────┐
  │ Meilisearch  │  (index for search)
  └──────┬───────┘
         │
         ▼
  mark_completed in crawl_queue

  ```

  ### Crawler fixes:
  ✅ UTF-8 panic: safe truncation with notice_core::truncate_utf8()
   - Tested with ASCII, French (è), CJK (能) characters
   - Applied in both worker.rs and content.rs

✅ Crawler noise: same-domain link filtering
   - Links to other domains are now dropped
   - Wikipedia special pages (Talk:, User:, Special:, etc.) are filtered
   - Queue should now have tens of URLs per crawl, not thousands
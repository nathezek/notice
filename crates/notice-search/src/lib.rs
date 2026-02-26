use meilisearch_sdk::client::Client;
use meilisearch_sdk::search::Selectors;
use notice_core::types::SearchResult;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

const DOCUMENTS_INDEX: &str = "documents";

/// Wrapper around the Meilisearch client.
#[derive(Clone)]
pub struct SearchClient {
    client: Client,
}

// ─── Meilisearch Document Types ───

/// What we SEND to Meilisearch (all indexed fields).
/// Used for direct sync and must match what MeiliBridge sends.
#[derive(Debug, Clone, Serialize)]
pub struct MeiliDocumentInput {
    pub id: Uuid,
    pub url: String,
    pub domain: String,
    pub title: Option<String>,
    pub raw_content: String,
    pub summary: Option<String>,
    pub status: String,
    pub quality_score: f64,
}

/// What we READ from Meilisearch search results.
/// Must match displayed_attributes — does NOT include raw_content.
#[derive(Debug, Clone, Deserialize)]
pub struct MeiliDocumentOutput {
    pub id: Uuid,
    pub url: String,
    pub domain: String,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub status: String,
    pub quality_score: f64,
}

impl SearchClient {
    /// Create a new Meilisearch client.
    pub fn new(url: &str, api_key: &str) -> Result<Self, notice_core::Error> {
        let client = Client::new(url, Some(api_key))
            .map_err(|e| notice_core::Error::Search(e.to_string()))?;

        tracing::info!("Meilisearch client created for {}", url);
        Ok(Self { client })
    }

    /// Health check.
    pub async fn health(&self) -> Result<(), notice_core::Error> {
        self.client
            .health()
            .await
            .map_err(|e| notice_core::Error::Search(e.to_string()))?;
        Ok(())
    }

    /// Configure the documents index with optimal settings.
    /// Idempotent — safe to call on every startup.
    pub async fn configure_index(&self) -> Result<(), notice_core::Error> {
        // Create index (ignore if already exists)
        let task = self
            .client
            .create_index(DOCUMENTS_INDEX, Some("id"))
            .await
            .map_err(|e| notice_core::Error::Search(e.to_string()))?;
        let _ = task.wait_for_completion(&self.client, None, None).await;

        let index = self.client.index(DOCUMENTS_INDEX);

        // Searchable: what fields are searched (order = priority)
        index
            .set_searchable_attributes(["title", "summary", "raw_content", "url", "domain"])
            .await
            .map_err(|e| notice_core::Error::Search(e.to_string()))?;

        // Displayed: what fields are returned in results
        // NOTE: raw_content is deliberately excluded — it's large and
        // we only need it for search matching, not for display.
        index
            .set_displayed_attributes(["id", "url", "domain", "title", "summary", "status", "quality_score"])
            .await
            .map_err(|e| notice_core::Error::Search(e.to_string()))?;

        // Filterable: for faceted search / filtering
        index
            .set_filterable_attributes(["domain", "status"])
            .await
            .map_err(|e| notice_core::Error::Search(e.to_string()))?;

        // Ranking rules
        index
            .set_ranking_rules([
                "words",
                "typo",
                "proximity",
                "attribute",
                "sort",
                "quality_score:desc",
                "exactness",
            ])
            .await
            .map_err(|e| notice_core::Error::Search(e.to_string()))?;

        tracing::info!("Meilisearch '{}' index configured", DOCUMENTS_INDEX);
        Ok(())
    }

    /// Configure synonyms for common abbreviations and aliases.
    /// Meilisearch handles these transparently during search.
    pub async fn configure_synonyms(&self) -> Result<(), notice_core::Error> {
        let index = self.client.index(DOCUMENTS_INDEX);

        let mut synonyms = std::collections::HashMap::new();

        // Helper to reduce boilerplate
        let mut add = |key: &str, vals: &[&str]| {
            synonyms.insert(
                key.to_string(),
                vals.iter().map(|v| v.to_string()).collect::<Vec<String>>(),
            );
        };

        // Programming languages
        add("js", &["javascript"]);
        add("javascript", &["js"]);
        add("ts", &["typescript"]);
        add("typescript", &["ts"]);
        add("py", &["python"]);
        add("python", &["py"]);
        add("rb", &["ruby"]);
        add("ruby", &["rb"]);
        add("cpp", &["c++"]);
        add("c++", &["cpp"]);
        add("golang", &["go"]);
        add("go", &["golang"]);

        // Databases
        add("pg", &["postgresql", "postgres"]);
        add("postgresql", &["postgres", "pg"]);
        add("postgres", &["postgresql", "pg"]);
        add("mongo", &["mongodb"]);
        add("mongodb", &["mongo"]);

        // Technologies
        add("k8s", &["kubernetes"]);
        add("kubernetes", &["k8s"]);
        add("wasm", &["webassembly"]);
        add("webassembly", &["wasm"]);
        add("ml", &["machine learning"]);
        add("ai", &["artificial intelligence"]);
        add("os", &["operating system"]);
        add("db", &["database"]);
        add("api", &["application programming interface"]);
        add("cli", &["command line interface"]);
        add("gui", &["graphical user interface"]);
        add("tui", &["terminal user interface"]);
        add("ui", &["user interface"]);
        add("ux", &["user experience"]);
        add("oop", &["object oriented programming"]);
        add("fp", &["functional programming"]);

        // Common abbreviations
        add("docs", &["documentation"]);
        add("doc", &["documentation"]);
        add("config", &["configuration"]);
        add("auth", &["authentication"]);
        add("env", &["environment"]);
        add("repo", &["repository"]);
        add("lib", &["library"]);
        add("pkg", &["package"]);
        add("deps", &["dependencies"]);
        add("dev", &["development"]);
        add("prod", &["production"]);
        add("impl", &["implementation"]);
        add("fn", &["function"]);
        add("func", &["function"]);
        add("var", &["variable"]);
        add("arg", &["argument"]);
        add("param", &["parameter"]);
        add("err", &["error"]);
        add("msg", &["message"]);
        add("async", &["asynchronous"]);
        add("sync", &["synchronous"]);

        let count = synonyms.len();

        index
            .set_synonyms(&synonyms)
            .await
            .map_err(|e| notice_core::Error::Search(e.to_string()))?;

        tracing::info!("Meilisearch synonyms configured ({} entries)", count);
        Ok(())
    }

    // ─── Write Operations (Direct Sync) ───

    /// Add or update documents in Meilisearch.
    /// Waits for the indexing task to complete (up to 30s).
    pub async fn add_documents(
        &self,
        docs: &[MeiliDocumentInput],
    ) -> Result<(), notice_core::Error> {
        if docs.is_empty() {
            return Ok(());
        }

        let index = self.client.index(DOCUMENTS_INDEX);

        let task = index
            .add_documents(docs, Some("id"))
            .await
            .map_err(|e| notice_core::Error::Search(e.to_string()))?;

        let task = task
            .wait_for_completion(
                &self.client,
                Some(Duration::from_millis(200)),
                Some(Duration::from_secs(30)),
            )
            .await
            .map_err(|e| notice_core::Error::Search(e.to_string()))?;

        // Check if the task succeeded
        if task.is_failure() {
            let error_msg = format!("Meilisearch indexing task failed: {:?}", task);
            tracing::error!("{}", error_msg);
            return Err(notice_core::Error::Search(error_msg));
        }

        tracing::debug!("Indexed {} document(s) in Meilisearch", docs.len());
        Ok(())
    }

    /// Add a single document to Meilisearch.
    pub async fn add_document(&self, doc: MeiliDocumentInput) -> Result<(), notice_core::Error> {
        self.add_documents(&[doc]).await
    }

    /// Delete a document from Meilisearch by ID.
    pub async fn delete_document(&self, id: Uuid) -> Result<(), notice_core::Error> {
        let index = self.client.index(DOCUMENTS_INDEX);

        let task = index
            .delete_document(&id.to_string())
            .await
            .map_err(|e| notice_core::Error::Search(e.to_string()))?;

        task.wait_for_completion(
            &self.client,
            Some(Duration::from_millis(200)),
            Some(Duration::from_secs(10)),
        )
        .await
        .map_err(|e| notice_core::Error::Search(e.to_string()))?;

        tracing::debug!("Deleted document {} from Meilisearch", id);
        Ok(())
    }

    /// Get the number of documents in the index.
    pub async fn document_count(&self) -> Result<usize, notice_core::Error> {
        let index = self.client.index(DOCUMENTS_INDEX);
        let stats = index
            .get_stats()
            .await
            .map_err(|e| notice_core::Error::Search(e.to_string()))?;
        Ok(stats.number_of_documents)
    }

    // ─── Search ───

    /// Search documents. Returns results with snippets.
    pub async fn search(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
    ) -> Result<(Vec<SearchResult>, usize), notice_core::Error> {
        let index = self.client.index(DOCUMENTS_INDEX);

        let results = index
            .search()
            .with_query(query)
            .with_limit(limit)
            .with_offset(offset)
            .with_show_ranking_score(true)
            // Request crops for both summary and raw_content to have backup context
            .with_attributes_to_crop(Selectors::Some(&[
                ("summary", Some(200)),
                ("raw_content", Some(300)),
            ]))
            .with_attributes_to_highlight(Selectors::Some(&["title", "summary"]))
            .execute::<MeiliDocumentOutput>()
            .await
            .map_err(|e| notice_core::Error::Search(e.to_string()))?;

        let total = results.estimated_total_hits.unwrap_or(0);

        let search_results: Vec<SearchResult> = results
            .hits
            .into_iter()
            .map(|hit| {
                let doc = hit.result;

                // Priority for snippet in search results:
                // 1. Crop of raw_content (metadata)
                // 2. Summary as fallback
                let snippet = hit.formatted_result
                    .as_ref()
                    .and_then(|f| {
                        f.get("raw_content")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                    })
                    .unwrap_or_else(|| {
                        doc.summary.clone().unwrap_or_else(|| "No preview available".to_string())
                    });

                SearchResult {
                    id: doc.id,
                    url: doc.url,
                    title: doc.title,
                    snippet,
                    score: hit.ranking_score,
                }
            })
            .collect();

        Ok((search_results, total))
    }
}

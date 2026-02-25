use meilisearch_sdk::client::Client;
use notice_core::types::SearchResult;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const DOCUMENTS_INDEX: &str = "documents";

/// Wrapper around the Meilisearch client.
#[derive(Clone)]
pub struct SearchClient {
    client: Client,
}

/// The shape of a document as stored in the Meilisearch index.
/// Must match what MeiliBridge syncs from the documents table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeiliDocument {
    pub id: Uuid,
    pub url: String,
    pub domain: String,
    pub title: Option<String>,
    pub raw_content: String,
    pub summary: Option<String>,
    pub status: String,
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
    /// Should be called once at startup (idempotent).
    pub async fn configure_index(&self) -> Result<(), notice_core::Error> {
        let index = self.client.index(DOCUMENTS_INDEX);

        // Set the primary key
        let task = self
            .client
            .create_index(DOCUMENTS_INDEX, Some("id"))
            .await
            .map_err(|e| notice_core::Error::Search(e.to_string()))?;

        // Wait for index creation (ignore AlreadyExists errors)
        let _ = task.wait_for_completion(&self.client, None, None).await;

        // Searchable attributes: what fields can be searched
        index
            .set_searchable_attributes(["title", "summary", "raw_content", "url", "domain"])
            .await
            .map_err(|e| notice_core::Error::Search(e.to_string()))?;

        // Displayed attributes: what fields are returned in results
        index
            .set_displayed_attributes(["id", "url", "domain", "title", "summary", "status"])
            .await
            .map_err(|e| notice_core::Error::Search(e.to_string()))?;

        // Filterable attributes: what fields can be used in filters
        index
            .set_filterable_attributes(["domain", "status"])
            .await
            .map_err(|e| notice_core::Error::Search(e.to_string()))?;

        // Sortable attributes
        index
            .set_sortable_attributes(["created_at"])
            .await
            .map_err(|e| notice_core::Error::Search(e.to_string()))?;

        // Ranking rules (Meilisearch defaults are good, but we customize)
        index
            .set_ranking_rules([
                "words",
                "typo",
                "proximity",
                "attribute",
                "sort",
                "exactness",
            ])
            .await
            .map_err(|e| notice_core::Error::Search(e.to_string()))?;

        tracing::info!("Meilisearch '{}' index configured", DOCUMENTS_INDEX);
        Ok(())
    }

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
            .with_attributes_to_crop(["summary", "raw_content"])
            .with_crop_length(200)
            .with_attributes_to_highlight(["title", "summary"])
            .execute::<MeiliDocument>()
            .await
            .map_err(|e| notice_core::Error::Search(e.to_string()))?;

        let total = results.estimated_total_hits.unwrap_or(0);

        let search_results: Vec<SearchResult> = results
            .hits
            .into_iter()
            .map(|hit| {
                let doc = hit.result;
                let snippet = doc
                    .summary
                    .clone()
                    .unwrap_or_else(|| truncate(&doc.raw_content, 200));

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

/// Truncate a string to max_len characters at a word boundary.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }
    match s[..max_len].rfind(' ') {
        Some(pos) => format!("{}...", &s[..pos]),
        None => format!("{}...", &s[..max_len]),
    }
}

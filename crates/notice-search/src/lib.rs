use meilisearch_sdk::client::Client;

/// Wrapper around the Meilisearch client.
#[derive(Clone)]
pub struct SearchClient {
    client: Client,
}

impl SearchClient {
    /// Create a new Meilisearch client.
    pub fn new(url: &str, api_key: &str) -> Result<Self, notice_core::Error> {
        let client = Client::new(url, Some(api_key))
            .map_err(|e| notice_core::Error::Search(e.to_string()))?;

        tracing::info!("Meilisearch client created for {}", url);

        Ok(Self { client })
    }

    /// Get a reference to the underlying Meilisearch client.
    /// Used for direct SDK operations until we build higher-level abstractions.
    pub fn inner(&self) -> &Client {
        &self.client
    }

    /// Health check — verify Meilisearch is reachable.
    pub async fn health(&self) -> Result<(), notice_core::Error> {
        self.client
            .health()
            .await
            .map_err(|e| notice_core::Error::Search(e.to_string()))?;
        Ok(())
    }
}

use meilisearch_sdk::client::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexDocument {
    pub id: String, // Meilisearch requires an 'id'. We will hash/encode the URL.
    pub url: String,
    pub title: String,
    pub cleaned_text: String,
    pub summary: Option<String>,
}

pub async fn init_indexer(url: &str, api_key: Option<&str>) -> Client {
    Client::new(url, api_key).expect("Meilisearch Client initialization failed")
}

pub async fn index_page(client: &Client, doc: &IndexDocument) -> Result<(), Box<dyn std::error::Error>> {
    let index = client.index("pages");
    
    // Add or replace the document in the 'pages' index
    index.add_documents(&[doc], Some("id")).await?;
    
    Ok(())
}

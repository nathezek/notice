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
    let client = Client::new(url, api_key).expect("Meilisearch Client initialization failed");
    
    // Ensure the index exists
    let _ = client.create_index("pages", Some("id")).await;
    
    client
}

pub async fn index_page(client: &Client, doc: &IndexDocument) -> Result<(), Box<dyn std::error::Error>> {
    let index = client.index("pages");
    
    // Add or replace the document in the 'pages' index
    index.add_documents(&[doc], Some("id")).await?;
    
    Ok(())
}

pub async fn search_index(client: &Client, query: &str) -> Result<Vec<IndexDocument>, Box<dyn std::error::Error>> {
    let index = client.index("pages");
    
    let results = index.search()
        .with_query(query)
        .with_limit(3)
        .execute::<IndexDocument>()
        .await?;

    let hits = results.hits.into_iter().map(|hit| hit.result).collect();
    Ok(hits)
}

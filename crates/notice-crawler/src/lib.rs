pub mod discovery;
pub mod links;
pub mod rate_limiter;
pub mod robots;
pub mod scraper_engine;
pub mod worker;

pub use worker::{CrawlerHandle, start_crawler};

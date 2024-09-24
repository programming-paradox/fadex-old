// src/main.rs

mod db;
mod parser;
mod crawler;

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use url::Url;
use crawler::crawl;
use db::initialize_db_pool;

#[tokio::main]
async fn main() {
    // Configuration
    let start_url = "https://discord.com".to_string();
    let base_url = Url::parse(&start_url).expect("Invalid start URL");
    
    // Initialize the database pool
    let db_pool = initialize_db_pool().await;

    // Initialize the visited URLs set
    let visited = Arc::new(Mutex::new(HashSet::new()));

    // Start crawling
    crawl(start_url, base_url, visited, db_pool).await;
}

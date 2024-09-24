use crate::db::save_to_db;
use crate::parser::{parse_html, extract_links, sanitize_link};
use bb8_postgres::PostgresConnectionManager;
use bb8::Pool;
use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use tokio::task;
use tokio::sync::Semaphore;
use url::Url;
use lazy_static::lazy_static;

lazy_static! {
    static ref SEMAPHORE: Arc<Semaphore> = Arc::new(Semaphore::new(MAX_CONCURRENT_TASKS));
}

const MAX_CONCURRENT_TASKS: usize = 100;

pub async fn crawl(
    start_url: String,
    base_url: Url,
    visited: Arc<Mutex<HashSet<String>>>,
    pool: Pool<PostgresConnectionManager<tokio_postgres::NoTls>>,
) {
    let queue = Arc::new(Mutex::new(VecDeque::new()));
    {
        let mut q = queue.lock().unwrap();
        q.push_back(start_url);
    }

    let mut handles = Vec::new();

    loop {
        let url_option = {
            let mut q = queue.lock().unwrap();
            q.pop_front()
        };

        let url = match url_option {
            Some(url) => url,
            None => {
                // Queue is empty, wait for new URLs to be enqueued
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                continue;
            }
        };

        // Check if the URL has already been visited
        {
            let mut v = visited.lock().unwrap();
            if v.contains(&url) {
                continue;
            }
            v.insert(url.clone());
        }

        // Acquire a permit before proceeding
        let permit = SEMAPHORE.clone().acquire_owned().await.unwrap();

        // Clone the necessary Arcs for the task
        let queue_clone = Arc::clone(&queue);
        let visited_clone = Arc::clone(&visited);
        let pool_clone = pool.clone();
        let base_url_clone = base_url.clone();

        // Spawn a new task to process the URL
        let handle = task::spawn(async move {
            // Fetch the page
            let body = match fetch_page(&url).await {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("Error fetching {}: {:?}", url, e);
                    drop(permit); // Release the permit
                    return;
                }
            };

            // Parse the HTML to extract title and description
            let (title, description) = parse_html(&body);

            // Save the extracted data to the database
            println!("Website Cloned");
            save_to_db(&pool_clone, &url, title, description).await;

            // Extract links from the page
            let links = extract_links(&body, &base_url_clone);

            // Enqueue new links
            {
                let mut q = queue_clone.lock().unwrap();
                for link in links {
                    if let Some(sanitized_link) = sanitize_link(&link) {
                        // Check if the link has already been visited
                        let already_visited = {
                            let v = visited_clone.lock().unwrap();
                            v.contains(&sanitized_link)
                        };
                        if !already_visited {
                            q.push_back(sanitized_link);
                        }
                    }
                }
            }

            drop(permit); // Release the permit
        });

        handles.push(handle);
    }

    // Await all spawned tasks to ensure completion
}

/// Fetches the content of the given URL.
///
/// # Arguments
///
/// * `url` - A string slice containing the URL to fetch.
///
/// # Returns
///
/// A `Result` containing the page content as a `String` or a `reqwest::Error`.
async fn fetch_page(url: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::get(url).await?;
    let content = response.text().await?;
    Ok(content)
}

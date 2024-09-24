// src/db.rs

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;

/// Initializes and returns a PostgreSQL connection pool.
pub async fn initialize_db_pool() -> Pool<PostgresConnectionManager<NoTls>> {
    let database_url = "host=localhost user=fahad dbname=fadex password=fahad";
    
    let manager = PostgresConnectionManager::new_from_stringlike(database_url, NoTls)
        .expect("Failed to create PostgresConnectionManager");

    Pool::builder()
        .build(manager)
        .await
        .expect("Failed to create pool")
}

pub async fn save_to_db(
    pool: &Pool<PostgresConnectionManager<NoTls>>,
    url: &str,
    title: Option<String>,
    description: Option<String>,
) {
    let client = pool.get().await.expect("Failed to get DB client");

    // Check if the URL already exists
    let row = client
        .query_opt("SELECT 1 FROM web_pages WHERE url = $1", &[&url])
        .await
        .expect("Failed to execute query");

    if let Some(_) = row {
        // URL exists, update the record
        let stmt = client
            .prepare("UPDATE web_pages SET title = $2, description = $3 WHERE url = $1")
            .await
            .expect("Failed to prepare statement");

        client
            .execute(
                &stmt,
                &[
                    &url,
                    &title.as_deref().unwrap_or(""),
                    &description.as_deref().unwrap_or(""),
                ],
            )
            .await
            .expect("Failed to execute statement");
    } else {
        // URL does not exist, insert a new record
        let stmt = client
            .prepare("INSERT INTO web_pages (url, title, description) VALUES ($1, $2, $3)")
            .await
            .expect("Failed to prepare statement");

        client
            .execute(
                &stmt,
                &[
                    &url,
                    &title.as_deref().unwrap_or(""),
                    &description.as_deref().unwrap_or(""),
                ],
            )
            .await
            .expect("Failed to execute statement");
    }
}
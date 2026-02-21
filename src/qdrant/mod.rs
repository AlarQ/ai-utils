mod qdrant_service;

pub use qdrant_service::*;

#[cfg(test)]
mod tests {
    use std::{env, time::Duration};

    use qdrant_client::Qdrant;

    #[tokio::test]
    async fn test() {
        dotenv::dotenv().ok();

        let Ok(url) = env::var("QDRANT_URL") else {
            println!("Skipping test: QDRANT_URL not set");
            return;
        };
        let Ok(api_key) = env::var("QDRANT_API_KEY") else {
            println!("Skipping test: QDRANT_API_KEY not set");
            return;
        };

        println!("Connecting to Qdrant at URL: {}", url);
        println!("Using API key: {}", api_key);

        let client = Qdrant::from_url(&url)
            .api_key(api_key)
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        // First try a simple health check
        match client.health_check().await {
            Ok(_) => println!("Health check successful"),
            Err(e) => println!("Health check failed: {:?}", e),
        }

        let collections_list = client.list_collections().await;
        let _ = dbg!(collections_list);
    }
}

use std::{collections::HashMap, env};

use async_trait::async_trait;
use qdrant_client::{
    qdrant::{
        CreateCollectionBuilder, Distance, PointStruct, SearchParamsBuilder, SearchPointsBuilder,
        UpsertPointsBuilder, VectorParamsBuilder,
    },
    Payload, Qdrant, QdrantError,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::error::Error;

/// Trait for embedding services that can generate vector embeddings
#[async_trait]
pub trait EmbeddingService: Send + Sync {
    /// Embed a single text into a vector
    async fn embed(&self, text: String) -> crate::Result<Vec<f32>>;

    /// Embed multiple texts into vectors
    async fn embed_batch(&self, texts: Vec<String>) -> crate::Result<Vec<Vec<f32>>>;
}

pub struct QdrantService {
    client: Qdrant,
    embedding_service: Box<dyn EmbeddingService>,
}

impl QdrantService {
    pub fn new() -> Result<Self, Error> {
        let url = env::var("QDRANT_URL")
            .map_err(|_| Error::Config("QDRANT_URL must be set".to_string()))?;
        let api_key = env::var("QDRANT_API_KEY")
            .map_err(|_| Error::Config("QDRANT_API_KEY must be set".to_string()))?;

        let client = Qdrant::from_url(&url)
            .api_key(api_key)
            .build()
            .map_err(|e| Error::Other(format!("Failed to create Qdrant client: {e}")))?;

        // Use OpenRouterService by default for embeddings
        let openrouter = crate::openrouter::OpenRouterService::new()?;

        Ok(Self {
            client,
            embedding_service: Box::new(openrouter),
        })
    }

    /// Create QdrantService with a custom embedding service
    pub fn with_embedding_service(service: Box<dyn EmbeddingService>) -> Result<Self, Error> {
        let url = env::var("QDRANT_URL")
            .map_err(|_| Error::Config("QDRANT_URL must be set".to_string()))?;
        let api_key = env::var("QDRANT_API_KEY")
            .map_err(|_| Error::Config("QDRANT_API_KEY must be set".to_string()))?;

        let client = Qdrant::from_url(&url)
            .api_key(api_key)
            .build()
            .map_err(|e| Error::Other(format!("Failed to create Qdrant client: {e}")))?;

        Ok(Self {
            client,
            embedding_service: service,
        })
    }

    pub async fn list_collections(&self) -> Result<Vec<String>, QdrantError> {
        let collections = self.client.list_collections().await?;
        Ok(collections
            .collections
            .into_iter()
            .map(|c| c.name)
            .collect())
    }

    pub async fn create_collection(
        &self,
        collection_name: &str,
        vector_size: u64,
    ) -> Result<(), QdrantError> {
        let _collection = self
            .client
            .create_collection(
                CreateCollectionBuilder::new(collection_name)
                    .vectors_config(VectorParamsBuilder::new(vector_size, Distance::Cosine)),
            )
            .await?;
        Ok(())
    }

    pub async fn upsert_point(
        &self,
        collection_name: &str,
        point: PointInput,
    ) -> Result<(), Error> {
        let vector = self.embedding_service.embed(point.text.clone()).await?;

        let payload: Payload = json!(point)
            .as_object()
            .ok_or_else(|| Error::Other("Failed to convert point to JSON object".to_string()))?
            .clone()
            .into();

        let point_id = point
            .id
            .parse::<u64>()
            .map_err(|e| Error::Other(format!("Invalid point ID '{}': {e}", point.id)))?;
        let points = vec![PointStruct::new(point_id, vector, payload)];

        self.client
            .upsert_points(UpsertPointsBuilder::new(collection_name, points))
            .await
            .map_err(|e| Error::Other(format!("Failed to upsert points: {e}")))?;

        Ok(())
    }
    pub async fn upsert_points(
        &self,
        collection_name: &str,
        points: Vec<PointInput>,
    ) -> Result<(), Error> {
        for point in points {
            self.upsert_point(collection_name, point).await?;
        }

        Ok(())
    }

    pub async fn search_points(
        &self,
        collection_name: String,
        query: String,
        limit: u64,
    ) -> Result<Vec<QueryOutput>, Error> {
        let vector = self.embedding_service.embed(query.clone()).await?;

        let points = self
            .client
            .search_points(
                SearchPointsBuilder::new(collection_name, vector, limit)
                    .with_payload(true)
                    .params(SearchParamsBuilder::default().hnsw_ef(128).exact(false)),
            )
            .await?
            .result
            .into_iter()
            .map(|p| {
                QueryOutput(
                    p.payload
                        .into_iter()
                        .map(|(k, v)| (k, v.to_string()))
                        .collect(),
                )
            })
            .collect();

        Ok(points)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PointInput {
    pub id: String,
    pub text: String,
    pub metadata: HashMap<String, String>,
}

impl PointInput {
    pub fn new(id: &str, text: &str, metadata: &HashMap<String, String>) -> Self {
        Self {
            id: id.to_string(),
            text: text.to_string(),
            metadata: metadata.clone(),
        }
    }
}

pub struct QueryOutput(pub HashMap<String, String>);

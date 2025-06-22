use std::{collections::HashMap, env};

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
use crate::openai::{AIService, OpenAIService};

#[derive(Debug, Clone)]
pub struct QdrantConfig {
    pub url: String,
    pub api_key: String,
}

impl QdrantConfig {
    pub fn from_env() -> Result<Self, Error> {
        let url = std::env::var("QDRANT_URL")
            .map_err(|_| Error::Config("QDRANT_URL must be set".to_string()))?;
        let api_key = std::env::var("QDRANT_API_KEY")
            .map_err(|_| Error::Config("QDRANT_API_KEY must be set".to_string()))?;
        Ok(Self { url, api_key })
    }
}

pub struct QdrantService {
    client: Qdrant,
    openai_service: OpenAIService,
}

impl QdrantService {
    pub fn new(config: QdrantConfig) -> Result<Self, Error> {
        let client = Qdrant::from_url(&config.url)
            .api_key(config.api_key)
            .build()
            .map_err(|e| Error::Other(format!("Failed to create Qdrant client: {}", e)))?;

        Ok(Self {
            client,
            openai_service: OpenAIService::new()?,
        })
    }

    pub async fn list_collections(&self) -> Result<Vec<String>, Error> {
        let collections = self
            .client
            .list_collections()
            .await
            .map_err(|e| Error::Other(format!("Failed to list collections: {}", e)))?;
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
    ) -> Result<(), Error> {
        let _collection = self
            .client
            .create_collection(
                CreateCollectionBuilder::new(collection_name)
                    .vectors_config(VectorParamsBuilder::new(vector_size, Distance::Cosine)),
            )
            .await
            .map_err(|e| {
                Error::Other(format!(
                    "Failed to create collection '{}': {}",
                    collection_name, e
                ))
            })?;
        Ok(())
    }

    pub async fn upsert_point(
        &self,
        collection_name: &str,
        point: PointInput,
    ) -> Result<(), Error> {
        let vector = self
            .openai_service
            .embed(point.text.clone())
            .await
            .map_err(|e| {
                Error::Other(format!(
                    "Failed to embed text for point '{}': {}",
                    point.id, e
                ))
            })?;

        let payload: Payload = json!(point)
            .as_object()
            .ok_or_else(|| Error::Other("Failed to serialize point to JSON object".to_string()))?
            .clone()
            .into();

        let point_id = point
            .id
            .parse::<u64>()
            .map_err(|e| Error::Other(format!("Invalid point ID '{}': {}", point.id, e)))?;

        let points = vec![PointStruct::new(point_id, vector, payload)];

        self.client
            .upsert_points(UpsertPointsBuilder::new(collection_name, points))
            .await
            .map_err(|e| {
                Error::Other(format!(
                    "Failed to upsert point '{}' in collection '{}': {}",
                    point.id, collection_name, e
                ))
            })?;

        Ok(())
    }

    pub async fn upsert_points(
        &self,
        collection_name: &str,
        points: Vec<PointInput>,
    ) -> Result<(), Error> {
        // Use batch implementation for better performance
        self.upsert_points_batch(collection_name, points).await
    }

    pub async fn upsert_points_batch(
        &self,
        collection_name: &str,
        points: Vec<PointInput>,
    ) -> Result<(), Error> {
        if points.is_empty() {
            return Ok(());
        }

        // Extract all texts for batch embedding
        let texts: Vec<String> = points.iter().map(|p| p.text.clone()).collect();

        // Batch embed all texts at once
        let vectors = self
            .openai_service
            .embed_batch(texts)
            .await
            .map_err(|e| Error::Other(format!("Failed to batch embed texts: {}", e)))?;

        if vectors.len() != points.len() {
            return Err(Error::Other(format!(
                "Embedding count mismatch: expected {}, got {}",
                points.len(),
                vectors.len()
            )));
        }

        // Create point structs with embedded vectors
        let point_structs: Result<Vec<PointStruct>, Error> = points
            .into_iter()
            .zip(vectors.into_iter())
            .map(|(point, vector)| {
                let payload: Payload = json!(point)
                    .as_object()
                    .ok_or_else(|| {
                        Error::Other("Failed to serialize point to JSON object".to_string())
                    })?
                    .clone()
                    .into();

                let point_id = point
                    .id
                    .parse::<u64>()
                    .map_err(|e| Error::Other(format!("Invalid point ID '{}': {}", point.id, e)))?;

                Ok(PointStruct::new(point_id, vector, payload))
            })
            .collect();

        let point_structs = point_structs?;

        // Batch upsert all points
        self.client
            .upsert_points(UpsertPointsBuilder::new(collection_name, point_structs))
            .await
            .map_err(|e| {
                Error::Other(format!(
                    "Failed to batch upsert points in collection '{}': {}",
                    collection_name, e
                ))
            })?;

        Ok(())
    }

    pub async fn search_points(
        &self,
        collection_name: String,
        query: String,
        limit: u64,
    ) -> Result<Vec<QueryOutput>, Error> {
        let vector = self
            .openai_service
            .embed(query.clone())
            .await
            .map_err(|e| Error::Other(format!("Failed to embed query '{}': {}", query, e)))?;

        let points = self
            .client
            .search_points(
                SearchPointsBuilder::new(collection_name.clone(), vector, limit)
                    .with_payload(true)
                    .params(SearchParamsBuilder::default().hnsw_ef(128).exact(false)),
            )
            .await
            .map_err(|e| {
                Error::Other(format!(
                    "Failed to search points in collection '{}': {}",
                    collection_name, e
                ))
            })?
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

    #[cfg(test)]
    pub async fn delete_collection(&self, collection_name: &str) -> Result<(), Error> {
        self.client
            .delete_collection(collection_name)
            .await
            .map_err(|e| {
                Error::Other(format!(
                    "Failed to delete collection '{}': {}",
                    collection_name, e
                ))
            })?;
        Ok(())
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

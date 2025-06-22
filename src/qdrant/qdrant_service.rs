use std::{collections::HashMap, env};

use qdrant_client::{
    qdrant::{
        CreateCollectionBuilder, Distance, PointStruct, ScalarQuantizationBuilder, SearchParamsBuilder, SearchPointsBuilder, UpsertPointsBuilder, VectorParamsBuilder
    },
    Payload, Qdrant, QdrantError,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::openai::{AIService, OpenAIService};

pub struct QdrantService {
    client: Qdrant,
    openai_service: OpenAIService,
}

impl QdrantService {
    pub fn new() -> Self {
        let url = env::var("QDRANT_URL").unwrap();
        let api_key = env::var("QDRANT_API_KEY").unwrap();

        let client = Qdrant::from_url(&url)
            .api_key(api_key)
            .build()
            .unwrap();

        Self {
            client,
            openai_service: OpenAIService::new(),
        }
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
    ) -> Result<(), QdrantError> {
        let vector = self.openai_service.embed(point.text.clone()).await.unwrap();

        let payload: Payload = json!(point).as_object().unwrap().clone().into();

        let points = vec![PointStruct::new(
            point.id.parse::<u64>().unwrap(),
            vector,
            payload,
        )];

        self.client
            .upsert_points(UpsertPointsBuilder::new(collection_name, points))
            .await?;

        Ok(())
    }
    pub async fn upsert_points(
        &self,
        collection_name: &str,
        points: Vec<PointInput>,
    ) -> Result<(), QdrantError> {
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
    ) -> Result<Vec<QueryOutput>, QdrantError> {
        let vector = self.openai_service.embed(query.clone()).await.unwrap();

        let points = self
            .client
            .search_points(SearchPointsBuilder::new(collection_name, vector, limit)
            .with_payload(true)
            .params(SearchParamsBuilder::default().hnsw_ef(128).exact(false))
            
        )
            .await
            .unwrap()
            .result
            .into_iter()
            .map(|p| QueryOutput(p.payload.into_iter().map(|(k, v)| (k, v.to_string())).collect()))
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

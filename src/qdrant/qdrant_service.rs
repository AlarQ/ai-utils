use std::{collections::HashMap, env};

use qdrant_client::{
    qdrant::{
        CreateCollectionBuilder, Distance, Filter, PointStruct, SearchParamsBuilder,
        SearchPointsBuilder, UpsertPointsBuilder, VectorParamsBuilder,
    },
    Payload, Qdrant, QdrantError,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::Error;

#[cfg(feature = "openai")]
use crate::openai::{AIService, OpenAIService};

// Constants for Qdrant configuration
pub const DEFAULT_HNSW_EF: u64 = 128;
pub const DEFAULT_SEARCH_LIMIT: u64 = 10;
pub const TEXT_EMBEDDING_3_LARGE_DIMENSION: u64 = 3072;

#[derive(Debug, Clone)]
pub struct QdrantConfig {
    pub url: String,
    pub api_key: String,
}

impl QdrantConfig {
    pub fn from_env() -> Result<Self, Error> {
        let mut url = std::env::var("QDRANT_URL")
            .map_err(|_| Error::Config("QDRANT_URL must be set".to_string()))?;
        let api_key = std::env::var("QDRANT_API_KEY")
            .map_err(|_| Error::Config("QDRANT_API_KEY must be set".to_string()))?;

        // Convert HTTPS to HTTP for cloud Qdrant to avoid compression issues
        if url.starts_with("https://") && url.contains("cloud.qdrant.io") {
            url = url.replace("https://", "http://");
        }

        Ok(Self { url, api_key })
    }
}

pub struct QdrantService {
    client: Qdrant,
    #[cfg(feature = "openai")]
    openai_service: OpenAIService,
}

impl QdrantService {
    pub fn new(config: QdrantConfig) -> Result<Self, Error> {
        let client = Qdrant::from_url(&config.url)
            .api_key(config.api_key)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| Error::Other(format!("Failed to create Qdrant client: {}", e)))?;

        Ok(Self {
            client,
            #[cfg(feature = "openai")]
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
        #[cfg(feature = "openai")]
        {
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
                .ok_or_else(|| {
                    Error::Other("Failed to serialize point to JSON object".to_string())
                })?
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

        #[cfg(not(feature = "openai"))]
        {
            Err(Error::Other(
                "OpenAI feature is required for upsert_point. Enable the 'openai' feature."
                    .to_string(),
            ))
        }
    }

    pub async fn upsert_points(
        &self,
        collection_name: &str,
        points: Vec<PointInput>,
    ) -> Result<BatchUpsertResult, Error> {
        self.upsert_points_batch(collection_name, points).await
    }

    pub async fn upsert_points_batch(
        &self,
        collection_name: &str,
        points: Vec<PointInput>,
    ) -> Result<BatchUpsertResult, Error> {
        #[cfg(feature = "openai")]
        {
            if points.is_empty() {
                return Ok(BatchUpsertResult {
                    successes: 0,
                    errors: vec![],
                });
            }

            let texts: Vec<String> = points.iter().map(|p| p.text.clone()).collect();
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

            let mut successes = 0;
            let mut errors = Vec::new();
            let mut point_structs = Vec::with_capacity(points.len());

            for (i, (point, vector)) in points.into_iter().zip(vectors.into_iter()).enumerate() {
                let payload: Result<Payload, Error> = json!(point)
                    .as_object()
                    .ok_or_else(|| {
                        Error::Other("Failed to serialize point to JSON object".to_string())
                    })
                    .map(|m| Payload::from(m.clone()));
                let point_id = point
                    .id
                    .parse::<u64>()
                    .map_err(|e| Error::Other(format!("Invalid point ID '{}': {}", point.id, e)));
                match (payload, point_id) {
                    (Ok(payload), Ok(point_id)) => {
                        point_structs.push(PointStruct::new(point_id, vector, payload));
                        successes += 1;
                    }
                    (Err(e), _) | (_, Err(e)) => {
                        errors.push((i, e));
                    }
                }
            }

            if !point_structs.is_empty() {
                if let Err(e) = self
                    .client
                    .upsert_points(UpsertPointsBuilder::new(collection_name, point_structs))
                    .await
                {
                    errors.push((
                        usize::MAX,
                        Error::Other(format!(
                            "Failed to batch upsert points in collection '{}': {}",
                            collection_name, e
                        )),
                    ));
                }
            }

            Ok(BatchUpsertResult { successes, errors })
        }

        #[cfg(not(feature = "openai"))]
        {
            Err(Error::Other(
                "OpenAI feature is required for upsert_points_batch. Enable the 'openai' feature."
                    .to_string(),
            ))
        }
    }

    pub async fn search_points(
        &self,
        collection_name: String,
        query: String,
        limit: u64,
    ) -> Result<Vec<QueryOutput>, Error> {
        #[cfg(feature = "openai")]
        {
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
                        .params(
                            SearchParamsBuilder::default()
                                .hnsw_ef(DEFAULT_HNSW_EF)
                                .exact(false),
                        ),
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

        #[cfg(not(feature = "openai"))]
        {
            Err(Error::Other(
                "OpenAI feature is required for search_points. Enable the 'openai' feature."
                    .to_string(),
            ))
        }
    }

    pub async fn upsert_point_with_vector(
        &self,
        collection_name: &str,
        point_id: u64,
        vector: Vec<f32>,
        payload: HashMap<String, String>,
    ) -> Result<(), Error> {
        let payload: Payload = json!(payload).as_object().unwrap().clone().into();
        let points = vec![PointStruct::new(point_id, vector, payload)];

        self.client
            .upsert_points(UpsertPointsBuilder::new(collection_name, points))
            .await
            .map_err(|e| {
                Error::Other(format!(
                    "Failed to upsert point '{}' in collection '{}': {}",
                    point_id, collection_name, e
                ))
            })?;

        Ok(())
    }

    pub async fn search_points_with_vector(
        &self,
        collection_name: String,
        vector: Vec<f32>,
        limit: u64,
    ) -> Result<Vec<QueryOutput>, Error> {
        let points = self
            .client
            .search_points(
                SearchPointsBuilder::new(collection_name.clone(), vector, limit)
                    .with_payload(true)
                    .params(
                        SearchParamsBuilder::default()
                            .hnsw_ef(DEFAULT_HNSW_EF)
                            .exact(false),
                    ),
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

    pub async fn update_collection(
        &self,
        builder: qdrant_client::qdrant::UpdateCollectionBuilder,
    ) -> Result<(), Error> {
        self.client
            .update_collection(builder)
            .await
            .map_err(|e| Error::Other(format!("Failed to update collection: {}", e)))?;
        Ok(())
    }

    pub async fn get_collection_info(
        &self,
        collection_name: &str,
    ) -> Result<qdrant_client::qdrant::CollectionInfo, Error> {
        let resp = self
            .client
            .collection_info(collection_name)
            .await
            .map_err(|e| {
                Error::Other(format!(
                    "Failed to get info for collection '{}': {}",
                    collection_name, e
                ))
            })?;
        resp.result.ok_or_else(|| {
            Error::Other(format!(
                "No info found for collection '{}': response missing result field",
                collection_name
            ))
        })
    }

    pub async fn health_check(&self) -> Result<(), Error> {
        self.client
            .health_check()
            .await
            .map_err(|e| Error::Other(format!("Qdrant health check failed: {}", e)))?;
        Ok(())
    }

    pub fn search_builder(&self, collection_name: impl Into<String>) -> QdrantSearchBuilder {
        QdrantSearchBuilder::new(self, collection_name)
    }
}

pub struct QdrantSearchBuilder<'a> {
    service: &'a QdrantService,
    collection_name: String,
    query_vector: Option<Vec<f32>>,
    query_text: Option<String>,
    limit: u64,
    hnsw_ef: Option<u64>,
    exact: Option<bool>,
    with_payload: bool,
    filter: Option<Filter>,
}

impl<'a> QdrantSearchBuilder<'a> {
    pub fn new(service: &'a QdrantService, collection_name: impl Into<String>) -> Self {
        Self {
            service,
            collection_name: collection_name.into(),
            query_vector: None,
            query_text: None,
            limit: DEFAULT_SEARCH_LIMIT,
            hnsw_ef: None,
            exact: None,
            with_payload: true,
            filter: None,
        }
    }

    pub fn query_vector(mut self, vector: Vec<f32>) -> Self {
        self.query_vector = Some(vector);
        self
    }

    pub fn query_text(mut self, text: impl Into<String>) -> Self {
        self.query_text = Some(text.into());
        self
    }

    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = limit;
        self
    }

    pub fn hnsw_ef(mut self, hnsw_ef: u64) -> Self {
        self.hnsw_ef = Some(hnsw_ef);
        self
    }

    pub fn exact(mut self, exact: bool) -> Self {
        self.exact = Some(exact);
        self
    }

    pub fn with_payload(mut self, with_payload: bool) -> Self {
        self.with_payload = with_payload;
        self
    }

    pub fn filter(mut self, filter: Filter) -> Self {
        self.filter = Some(filter);
        self
    }

    pub async fn search(self) -> Result<Vec<QueryOutput>, Error> {
        let vector = if let Some(vector) = self.query_vector {
            Some(vector)
        } else if let Some(text) = &self.query_text {
            #[cfg(feature = "openai")]
            {
                Some(
                    self.service
                        .openai_service
                        .embed(text.clone())
                        .await
                        .map_err(|e| Error::Other(format!("Failed to embed query text: {}", e)))?,
                )
            }
            #[cfg(not(feature = "openai"))]
            {
                return Err(Error::Other(
                    "OpenAI feature is required for text queries. Enable the 'openai' feature."
                        .to_string(),
                ));
            }
        } else {
            return Err(Error::Other(
                "Either query_vector or query_text must be set".to_string(),
            ));
        };

        let mut builder =
            SearchPointsBuilder::new(self.collection_name.clone(), vector.unwrap(), self.limit)
                .with_payload(self.with_payload);

        let mut params = SearchParamsBuilder::default();
        if let Some(hnsw_ef) = self.hnsw_ef {
            params = params.hnsw_ef(hnsw_ef);
        }
        if let Some(exact) = self.exact {
            params = params.exact(exact);
        }
        builder = builder.params(params);
        if let Some(filter) = self.filter {
            builder = builder.filter(filter);
        }

        let points = self
            .service
            .client
            .search_points(builder)
            .await
            .map_err(|e| {
                Error::Other(format!(
                    "Failed to search points in collection '{}': {}",
                    self.collection_name, e
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

#[derive(Debug, Clone)]
pub struct QueryOutput(pub HashMap<String, String>);

#[derive(Debug)]
pub struct BatchUpsertResult {
    pub successes: usize,
    pub errors: Vec<(usize, Error)>,
}

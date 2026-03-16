use std::collections::{BTreeMap, BTreeSet};

use async_trait::async_trait;
use chrono::Utc;
use sha2::{Digest, Sha256};
use thiserror::Error;
use url::Url;

use crate::domain::{
    ConnectorCapability, IngestionMethod, OperationalMode, SourceRecord, TrustLevel,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrawlRequest {
    pub connector_id: String,
    pub url: Url,
    pub mode: OperationalMode,
    pub entity_tags: Vec<String>,
    pub market_tags: Vec<String>,
    pub strategy_relevance_tags: Vec<String>,
    pub seed_content: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrawledSource {
    pub record: SourceRecord,
    pub content: String,
}

#[derive(Debug, Error)]
pub enum ConnectorError {
    #[error("host is not approved: {0}")]
    HostNotApproved(String),
    #[error("connector is not registered: {0}")]
    UnknownConnector(String),
}

#[derive(Debug, Default, Clone)]
pub struct ConnectorCapabilityRegistry {
    connectors: BTreeMap<String, ConnectorCapability>,
}

impl ConnectorCapabilityRegistry {
    pub fn register(&mut self, capability: ConnectorCapability) {
        self.connectors.insert(capability.connector_id.clone(), capability);
    }

    pub fn get(&self, connector_id: &str) -> Option<&ConnectorCapability> {
        self.connectors.get(connector_id)
    }
}

#[async_trait]
pub trait ApprovedSourceCrawler: Send + Sync {
    async fn crawl(&self, request: CrawlRequest) -> Result<CrawledSource, ConnectorError>;
}

#[derive(Clone)]
pub struct StaticSourceCrawler {
    approved_hosts: BTreeSet<String>,
    registry: ConnectorCapabilityRegistry,
}

impl StaticSourceCrawler {
    pub fn new(
        approved_hosts: impl IntoIterator<Item = impl Into<String>>,
        registry: ConnectorCapabilityRegistry,
    ) -> Self {
        Self { approved_hosts: approved_hosts.into_iter().map(Into::into).collect(), registry }
    }
}

#[async_trait]
impl ApprovedSourceCrawler for StaticSourceCrawler {
    async fn crawl(&self, request: CrawlRequest) -> Result<CrawledSource, ConnectorError> {
        let host = request.url.host_str().unwrap_or_default().to_string();
        if !self.approved_hosts.contains(&host) {
            return Err(ConnectorError::HostNotApproved(host));
        }
        let capability = self
            .registry
            .get(&request.connector_id)
            .ok_or_else(|| ConnectorError::UnknownConnector(request.connector_id.clone()))?;

        let content = request.seed_content.unwrap_or_else(|| {
            format!(
                "approved crawler snapshot from {} using scopes {:?}",
                request.url, capability.scopes
            )
        });

        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let content_hash = format!("{:x}", hasher.finalize());

        Ok(CrawledSource {
            record: SourceRecord {
                id: format!("src-{}", &content_hash[..12]),
                connector_id: request.connector_id,
                source_uri: request.url.to_string(),
                source_name: host,
                timestamp: Utc::now(),
                trust_level: TrustLevel::High,
                ingestion_method: IngestionMethod::ScheduledCrawl,
                content_hash,
                freshness_score: 1.0,
                entity_tags: request.entity_tags,
                market_tags: request.market_tags,
                strategy_relevance_tags: request.strategy_relevance_tags,
                metadata: BTreeMap::from([
                    ("mode".to_string(), format!("{:?}", request.mode)),
                    ("session_isolation".to_string(), capability.session_isolation.clone()),
                ]),
            },
            content,
        })
    }
}

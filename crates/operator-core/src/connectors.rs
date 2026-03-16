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
#[serde(deny_unknown_fields)]
pub struct CrawlRequest {
    pub connector_id: String,
    pub url: Url,
    pub mode: OperationalMode,
    pub entity_tags: Vec<String>,
    pub market_tags: Vec<String>,
    pub strategy_relevance_tags: Vec<String>,
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

        let content = format!(
            "approved crawler snapshot from {} using scopes {:?}",
            request.url, capability.scopes
        );

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

#[cfg(test)]
mod tests {
    use serde_json::json;
    use url::Url;

    use crate::domain::{ConnectorCapability, ConnectorClass};

    use super::{
        ApprovedSourceCrawler, ConnectorCapabilityRegistry, ConnectorError, CrawlRequest,
        StaticSourceCrawler,
    };

    fn registry() -> ConnectorCapabilityRegistry {
        let mut registry = ConnectorCapabilityRegistry::default();
        registry.register(ConnectorCapability {
            connector_id: "approved-http".to_string(),
            class: ConnectorClass::ReadOnlyResearch,
            scopes: vec!["news.read".to_string()],
            rate_limit_per_minute: 30,
            dry_run_supported: true,
            session_isolation: "read-only-profile".to_string(),
        });
        registry
    }

    fn request(url: &str) -> CrawlRequest {
        CrawlRequest {
            connector_id: "approved-http".to_string(),
            url: Url::parse(url).expect("valid url"),
            mode: crate::domain::OperationalMode::Research,
            entity_tags: vec!["btc".to_string()],
            market_tags: vec!["crypto".to_string()],
            strategy_relevance_tags: vec!["momentum".to_string()],
        }
    }

    #[tokio::test]
    async fn crawler_rejects_unapproved_hosts() {
        let crawler = StaticSourceCrawler::new(["example.com"], registry());
        let error = crawler
            .crawl(request("https://unapproved.example.org/filing"))
            .await
            .expect_err("host must be rejected");

        assert!(
            matches!(error, ConnectorError::HostNotApproved(host) if host == "unapproved.example.org")
        );
    }

    #[tokio::test]
    async fn crawler_uses_approved_snapshot_metadata() {
        let crawler = StaticSourceCrawler::new(["example.com"], registry());
        let response = crawler
            .crawl(request("https://example.com/market-snapshot"))
            .await
            .expect("approved host should crawl");

        assert_eq!(response.record.connector_id, "approved-http");
        assert_eq!(response.record.trust_level, crate::domain::TrustLevel::High);
        assert_eq!(
            response.record.metadata.get("session_isolation"),
            Some(&"read-only-profile".to_string())
        );
        assert!(response.content.contains("approved crawler snapshot"));
        assert!(response.content.contains("https://example.com/market-snapshot"));
    }

    #[test]
    fn crawl_request_rejects_seed_content_in_json() {
        let error = serde_json::from_value::<CrawlRequest>(json!({
            "connectorId": "approved-http",
            "url": "https://example.com/market-snapshot",
            "mode": "research",
            "entityTags": [],
            "marketTags": [],
            "strategyRelevanceTags": [],
            "seedContent": "pwned"
        }))
        .expect_err("unknown fields must be rejected");

        assert!(error.to_string().contains("unknown field"));
    }
}

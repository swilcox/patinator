use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::cmp::Ordering;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct FieldDefaults {
    #[serde(default = "default_version_field")]
    pub version_field: String,
    #[serde(default = "default_deploy_time_field")]
    pub deploy_time_field: String,
}

fn default_version_field() -> String {
    "version".to_string()
}

fn default_deploy_time_field() -> String {
    "deployment_time".to_string()
}

#[derive(Debug, Deserialize, Clone)]
pub struct Service {
    pub name: String,
    pub tags: Vec<String>,
    #[serde(default)]
    pub field_mappings: FieldMappings,
    pub environments: Vec<Environment>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct FieldMappings {
    pub version_field: Option<String>,
    pub deploy_time_field: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Environment {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionResponse {
    pub version: String,
    #[serde(default)]
    pub deployment_time: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct VersionInfo {
    pub service_name: String,
    pub service_tags: Vec<String>,
    pub env_name: String,
    pub version: String,
    pub deployment_time: Option<DateTime<Utc>>,
}

impl PartialOrd for VersionInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for VersionInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.service_name
            .cmp(&other.service_name)
            .then(self.env_name.cmp(&other.env_name))
    }
}

impl PartialEq for VersionInfo {
    fn eq(&self, other: &Self) -> bool {
        self.service_name == other.service_name && self.env_name == other.env_name
    }
}

impl Eq for VersionInfo {}

#[derive(Debug, Serialize, Deserialize)]
pub struct DynamicVersionResponse {
    #[serde(flatten)]
    pub fields: serde_json::Map<String, serde_json::Value>,
}

use serde::Deserialize;
use std::path::Path;
use anyhow::Result;
use crate::types::Service;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub services: Vec<Service>,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }
}
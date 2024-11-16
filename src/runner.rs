use crate::config::Config;
use crate::types::*;
use anyhow::{Result, Context, anyhow};
use chrono::{DateTime, Utc};
use std::time::Duration;
use futures::future::join_all;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use std::collections::HashMap;
use tokio::task::JoinHandle;

pub struct VersionChecker {
    config: Config,
    client: reqwest::Client,
}

impl VersionChecker {
    pub fn new(config: Config) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { config, client }
    }

    pub async fn check_all(&self) -> Result<()> {
        let multi_progress = MultiProgress::new();
        let total_checks = self.config.services.iter()
            .map(|s| s.environments.len())
            .sum::<usize>();

        let main_progress = multi_progress.add(ProgressBar::new(total_checks as u64));
        main_progress.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} checks ({eta})")?
                .progress_chars("#>-")
        );

        let mut service_progress = HashMap::new();
        for service in &self.config.services {
            let service_bar = multi_progress.add(ProgressBar::new(service.environments.len() as u64));
            service_bar.set_style(
                ProgressStyle::default_bar()
                    .template("{prefix:.bold.dim} {spinner:.green} [{bar:20.cyan/blue}] {pos}/{len}")?
                    .progress_chars("#>-")
            );
            service_bar.set_prefix(format!("{:<20}", service.name));
            service_progress.insert(service.name.clone(), service_bar);
        }

        // Process all services in parallel
        let version_futures: Vec<JoinHandle<Result<Vec<VersionInfo>>>> = self.config
            .services
            .iter()
            .map(|service| {
                let client = self.client.clone();
                let service = service.clone();
                let main_progress = main_progress.clone();
                let service_progress = service_progress.get(&service.name).unwrap().clone();
                let defaults = self.config.defaults.clone();

                tokio::spawn(async move {
                    let mut version_infos = Vec::new();

                    let env_futures: Vec<JoinHandle<Result<VersionInfo>>> = service
                        .environments
                        .iter()
                        .map(|env| {
                            let client = client.clone();
                            let env = env.clone();
                            let service_name = service.name.clone();
                            let service_tags = service.tags.clone();
                            let service = service.clone();
                            let defaults = defaults.clone();
                            
                            tokio::spawn(async move {
                                let version_response = fetch_version_info(&client, &service, &env, &defaults).await
                                    .with_context(|| format!("Failed to fetch version for {}/{}", service_name, env.name))?;
                                
                                Ok(VersionInfo {
                                    service_name: service_name.clone(),
                                    service_tags: service_tags.clone(),
                                    env_name: env.name.clone(),
                                    version: version_response.version,
                                    deployment_time: version_response.deployment_time,
                                })
                            })
                        })
                        .collect();

                    let results = join_all(env_futures).await;
                    for result in results {
                        match result {
                            Ok(Ok(info)) => version_infos.push(info),
                            Ok(Err(e)) => eprintln!("Error fetching version: {}", e),
                            Err(e) => eprintln!("Task failed: {}", e),
                        }
                        main_progress.inc(1);
                        service_progress.inc(1);
                    }

                    version_infos.sort_by(|a, b| a.env_name.cmp(&b.env_name));
                    Ok(version_infos)
                })
            })
            .collect();

        let mut all_version_infos = Vec::new();
        let results = join_all(version_futures).await;
        for result in results {
            match result {
                Ok(Ok(infos)) => all_version_infos.extend(infos),
                Ok(Err(e)) => eprintln!("Error processing service: {}", e),
                Err(e) => eprintln!("Service task failed: {}", e),
            }
        }

        all_version_infos.sort();

        // Clear progress bars
        main_progress.finish_and_clear();
        for bar in service_progress.values() {
            bar.finish_and_clear();
        }

        // Drop the MultiProgress before printing results
        drop(multi_progress);

        // Print results
        let mut current_service = String::new();
        for info in all_version_infos {
            if current_service != info.service_name {
                println!("\nService: {} (tags: {})", 
                        info.service_name,
                        info.service_tags.join(", "));
                current_service = info.service_name;
            }

            print!("  {}: v{}", info.env_name, info.version);
            if let Some(deploy_time) = info.deployment_time {
                let duration = Utc::now().signed_duration_since(deploy_time);
                print!(" (deployed {} days ago)", duration.num_days());
            }
            println!();
        }

        Ok(())
    }
}

async fn fetch_version_info(
    client: &reqwest::Client, 
    service: &Service, 
    env: &Environment,
    defaults: &FieldDefaults
) -> Result<VersionResponse> {
    let response = client.get(&env.url).send().await?;
    let dynamic_response: DynamicVersionResponse = response.json().await?;
    
    // Determine field names to use
    let version_field = service.field_mappings.version_field
        .as_ref()
        .unwrap_or(&defaults.version_field);
        
    let deploy_time_field = service.field_mappings.deploy_time_field
        .as_ref()
        .unwrap_or(&defaults.deploy_time_field);

    // Extract version
    let version = dynamic_response.fields.get(version_field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing or invalid version field: {}", version_field))?
        .to_string();

    // Extract deployment time if present
    let deployment_time = dynamic_response.fields.get(deploy_time_field)
        .and_then(|v| v.as_str())
        .and_then(|ts| DateTime::parse_from_rfc3339(ts).ok())
        .map(|dt| dt.with_timezone(&Utc));

    Ok(VersionResponse {
        version,
        deployment_time,
    })
}
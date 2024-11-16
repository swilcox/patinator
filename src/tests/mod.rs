use crate::{
    config::Config,
    types::{Service, Environment, VersionResponse, VersionInfo, FieldDefaults, FieldMappings},
    runner::VersionChecker,
};
use chrono::{DateTime, Utc, TimeZone};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path}
};
use std::path::PathBuf;
use serde_json::json;

#[tokio::test]
async fn test_config_loading() {
    let config_content = r#"
services:
  - name: test-service
    tags: [test]
    environments:
      - name: dev
        url: http://dev.example.com/version
      - name: prod
        url: http://prod.example.com/version
"#;
    
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("test-config.yaml");
    std::fs::write(&config_path, config_content).unwrap();
    
    let config = Config::from_file(config_path).unwrap();
    
    assert_eq!(config.services.len(), 1);
    assert_eq!(config.services[0].name, "test-service");
    assert_eq!(config.services[0].tags, vec!["test"]);
    assert_eq!(config.services[0].environments.len(), 2);
}

#[tokio::test]
async fn test_custom_field_names() {
    let mock_server = MockServer::start().await;
    
    // Mock response with custom field names
    Mock::given(method("GET"))
        .and(path("/custom/version"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "ver": "1.2.3",
                "deploy_ts": "2024-01-01T00:00:00Z"
            })))
        .mount(&mock_server)
        .await;
    
    let config = Config {
        defaults: FieldDefaults {
            version_field: "version".to_string(),
            deploy_time_field: "deployment_time".to_string(),
        },
        services: vec![
            Service {
                name: "custom-service".to_string(),
                tags: vec!["test".to_string()],
                field_mappings: FieldMappings {
                    version_field: Some("ver".to_string()),
                    deploy_time_field: Some("deploy_ts".to_string()),
                },
                environments: vec![
                    Environment {
                        name: "dev".to_string(),
                        url: format!("{}/custom/version", mock_server.uri()),
                    },
                ],
            },
        ],
    };
    
    let checker = VersionChecker::new(config);
    let result = checker.check_all().await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_default_field_names() {
    let mock_server = MockServer::start().await;
    
    // Mock response with default field names
    Mock::given(method("GET"))
        .and(path("/default/version"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "version": "1.2.3",
                "deployment_time": "2024-01-01T00:00:00Z"
            })))
        .mount(&mock_server)
        .await;
    
    let config = Config {
        defaults: FieldDefaults::default(),
        services: vec![
            Service {
                name: "default-service".to_string(),
                tags: vec!["test".to_string()],
                field_mappings: FieldMappings::default(),
                environments: vec![
                    Environment {
                        name: "dev".to_string(),
                        url: format!("{}/default/version", mock_server.uri()),
                    },
                ],
            },
        ],
    };
    
    let checker = VersionChecker::new(config);
    let result = checker.check_all().await;
    
    assert!(result.is_ok());
}

#[test]
fn test_config_deserialization() {
    let yaml = r#"
defaults:
  version_field: "ver"
  deploy_time_field: "deploy_timestamp"
services:
  - name: service-a
    tags: [web]
    field_mappings:
      version_field: "v"
      deploy_time_field: "deployed_at"
    environments:
      - name: dev
        url: "http://dev.example.com/version"
  - name: service-b
    tags: [api]
    environments:
      - name: prod
        url: "http://prod.example.com/version"
"#;

    let config: Config = serde_yaml::from_str(yaml).unwrap();
    
    assert_eq!(config.defaults.version_field, "ver");
    assert_eq!(config.defaults.deploy_time_field, "deploy_timestamp");
    assert_eq!(config.services[0].field_mappings.version_field, Some("v".to_string()));
    assert!(config.services[1].field_mappings.version_field.is_none());
}

#[tokio::test]
async fn test_error_handling() {
    let mock_server = MockServer::start().await;
    
    // Mock a 404 response
    Mock::given(method("GET"))
        .and(path("/error/404"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    // Mock invalid JSON response
    Mock::given(method("GET"))
        .and(path("/error/invalid-json"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string("not json"))
        .mount(&mock_server)
        .await;

    // Mock missing required field
    Mock::given(method("GET"))
        .and(path("/error/missing-field"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "not_version": "1.2.3"
            })))
        .mount(&mock_server)
        .await;

    let config = Config {
        defaults: FieldDefaults::default(),
        services: vec![
            Service {
                name: "error-service".to_string(),
                tags: vec!["test".to_string()],
                field_mappings: FieldMappings::default(),
                environments: vec![
                    Environment {
                        name: "not-found".to_string(),
                        url: format!("{}/error/404", mock_server.uri()),
                    },
                    Environment {
                        name: "invalid-json".to_string(),
                        url: format!("{}/error/invalid-json", mock_server.uri()),
                    },
                    Environment {
                        name: "missing-field".to_string(),
                        url: format!("{}/error/missing-field", mock_server.uri()),
                    },
                ],
            },
        ],
    };
    
    let checker = VersionChecker::new(config);
    let result = checker.check_all().await;
    
    // The overall operation should complete, but we should see error output
    assert!(result.is_ok());
}

#[test]
fn test_version_info_ordering() {
    let info1 = VersionInfo {
        service_name: "service-a".to_string(),
        service_tags: vec![],
        env_name: "dev".to_string(),
        version: "1.0.0".to_string(),
        deployment_time: None,
    };
    
    let info2 = VersionInfo {
        service_name: "service-a".to_string(),
        service_tags: vec![],
        env_name: "prod".to_string(),
        version: "1.0.0".to_string(),
        deployment_time: None,
    };
    
    let info3 = VersionInfo {
        service_name: "service-b".to_string(),
        service_tags: vec![],
        env_name: "dev".to_string(),
        version: "1.0.0".to_string(),
        deployment_time: None,
    };
    
    // Test ordering by service name
    assert!(info1 < info3);
    
    // Test ordering by env name within same service
    assert!(info1 < info2);
}

#[tokio::test]
async fn test_invalid_deployment_time() {
    let mock_server = MockServer::start().await;
    
    // Mock response with invalid deployment time format
    Mock::given(method("GET"))
        .and(path("/invalid/datetime"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "version": "1.2.3",
                "deployment_time": "not-a-date"
            })))
        .mount(&mock_server)
        .await;
    
    let config = Config {
        defaults: FieldDefaults::default(),
        services: vec![
            Service {
                name: "datetime-service".to_string(),
                tags: vec!["test".to_string()],
                field_mappings: FieldMappings::default(),
                environments: vec![
                    Environment {
                        name: "dev".to_string(),
                        url: format!("{}/invalid/datetime", mock_server.uri()),
                    },
                ],
            },
        ],
    };
    
    let checker = VersionChecker::new(config);
    let result = checker.check_all().await;
    
    // Should complete successfully but deployment time should be None
    assert!(result.is_ok());
}

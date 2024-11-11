use crate::{
    config::Config,
    types::{Service, Environment, VersionResponse},
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
async fn test_version_response_parsing() {
    let json_str = r#"{
        "version": "1.2.3",
        "deployment_time": "2024-01-01T00:00:00Z"
    }"#;
    
    let response: VersionResponse = serde_json::from_str(json_str).unwrap();
    
    assert_eq!(response.version, "1.2.3");
    assert_eq!(
        response.deployment_time.unwrap(),
        Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap()
    );
}

#[tokio::test]
async fn test_version_checker_integration() {
    // Start a mock server
    let mock_server = MockServer::start().await;
    
    // Create mock responses
    Mock::given(method("GET"))
        .and(path("/dev/version"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "version": "1.2.3",
                "deployment_time": "2024-01-01T00:00:00Z"
            })))
        .mount(&mock_server)
        .await;
    
    Mock::given(method("GET"))
        .and(path("/prod/version"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({
                "version": "1.2.2",
                "deployment_time": "2023-12-25T00:00:00Z"
            })))
        .mount(&mock_server)
        .await;
    
    // Create test config
    let config = Config {
        services: vec![
            Service {
                name: "test-service".to_string(),
                tags: vec!["test".to_string()],
                environments: vec![
                    Environment {
                        name: "dev".to_string(),
                        url: format!("{}/dev/version", mock_server.uri()),
                    },
                    Environment {
                        name: "prod".to_string(),
                        url: format!("{}/prod/version", mock_server.uri()),
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
fn test_version_info_ordering() {
    use crate::types::VersionInfo;
    
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
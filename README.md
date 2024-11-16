# Patinator

A Rust CLI tool for tracking software versions across multiple environments. Like patina forms on copper over time, Patinator helps you monitor how your services age and change across environments.

## Features

- Parallel version checking of multiple services and environments
- Real-time progress indication with ETA
- Configurable field names for version and deployment time
- Support for different JSON response formats per service
- Deployment time tracking and age calculation
- Service tagging support

## Installation

```bash
cargo build --release
```

The binary will be available at `target/release/patinator`

## Usage

```bash
patinator --config config.yaml
```

## Configuration

The configuration file is in YAML format and supports the following structure:

```yaml
# Global defaults for field names
defaults:
  version_field: "version"        # Default field name for version
  deploy_time_field: "deployment_time"  # Default field name for deployment timestamp

services:
  - name: auth-service
    tags: [auth, security]
    # Optional: Override field names for this specific service
    field_mappings:
      version_field: "v"          # Use "v" instead of "version" for this service
      deploy_time_field: "deploy_ts"  # Use "deploy_ts" instead of "deployment_time"
    environments:
      - name: dev
        url: "http://dev.example.com/auth/version"
      - name: prod
        url: "http://prod.example.com/auth/version"

  - name: billing-service
    tags: [payments]
    # No field_mappings specified - will use defaults
    environments:
      - name: dev
        url: "http://dev.example.com/billing/version"
      - name: qa
        url: "http://qa.example.com/billing/version"
      - name: prod
        url: "http://prod.example.com/billing/version"
```

### Field Names Configuration

Patinator is flexible in handling different JSON response formats from version endpoints:

1. **Global Defaults**: Set default field names in the `defaults` section:
   ```yaml
   defaults:
     version_field: "version"
     deploy_time_field: "deployment_time"
   ```

2. **Service-Specific Overrides**: Override field names for specific services:
   ```yaml
   field_mappings:
     version_field: "v"
     deploy_time_field: "deploy_ts"
   ```

### Expected Response Formats

Patinator can handle various JSON response formats. Examples:

Default format:
```json
{
  "version": "1.2.3",
  "deployment_time": "2024-01-01T00:00:00Z"
}
```

Custom field names:
```json
{
  "v": "1.2.3",
  "deploy_ts": "2024-01-01T00:00:00Z"
}
```

Minimal format (deployment time is optional):
```json
{
  "version": "1.2.3"
}
```

### Service Tags

Services can be tagged for better organization and filtering:
```yaml
services:
  - name: auth-service
    tags: [auth, security]
    # ...
```

## Output Format

Patinator provides a clear, organized output:

```
Service: auth-service (tags: auth, security)
  dev: v1.2.3 (deployed 5 days ago)
  prod: v1.2.2 (deployed 15 days ago)

Service: billing-service (tags: payments)
  dev: v2.0.1 (deployed 3 days ago)
  qa: v2.0.1 (deployed 2 days ago)
  prod: v2.0.0 (deployed 10 days ago)
```

## Error Handling

- Connection failures are reported but don't stop the overall execution
- Invalid JSON responses are handled gracefully
- Missing required fields are reported with clear error messages
- Invalid datetime formats in deployment timestamps are ignored

## Progress Indication

The tool shows progress both overall and per-service:
```
[#########>----------------------] 12/40 checks (ETA 8s)
auth-service      [####>-----------] 4/10
billing-service   [######>---------] 6/10
```

## Development

### Running Tests

```bash
cargo test
```

### Building in Debug Mode

```bash
cargo build
```

### Running with Debug Output

```bash
RUST_LOG=debug cargo run -- --config config.yaml
```

## Why "Patinator"?

Patina is a film or coating that develops on the surface of metals over time through oxidation. Similarly, Patinator helps you track how your software versions "age" and change across different environments. The name is also a playful nod to Rust, both the programming language and the chemical process.

## License

MIT License
# Patinator

A command-line tool written in Rust for tracking and comparing versions across multiple services and environments.

## Features

- Track versions across multiple services simultaneously
- Support for multiple environments per service (e.g., production, staging)
- Service tagging for better organization
- Version comparison across environments
- Deployment time tracking
- YAML-based configuration

## Installation

Ensure you have Rust and Cargo installed on your system. Then:

```bash
cargo install patinator
```

Or clone and build from source:

```bash
git clone https://github.com/yourusername/patinator.git
cd patinator
cargo build --release
```

## Usage

Run patinator by providing a path to your services configuration file:

```bash
patinator --config services.yaml
```

### Configuration Format

Create a YAML file defining your services and environments. Example:

```yaml
services:
  - name: my-service
    tags: [backend, api]
    environments:
      - name: production
        url: https://api.myservice.com/version
      - name: staging
        url: https://staging.myservice.com/version
  - name: another-service
    tags: [frontend]
    environments:
      - name: production
        url: https://another.com/version
```

Each service configuration requires:
- `name`: Service identifier
- `tags`: List of tags for grouping/organizing services
- `environments`: List of environments to track
  - `name`: Environment name
  - `url`: Endpoint URL that returns version information

## Dependencies

- serde: Serialization/deserialization
- tokio: Async runtime
- reqwest: HTTP client
- clap: Command line argument parsing
- chrono: DateTime handling
- anyhow: Error handling

## License

This project is licensed under the MIT License - see the LICENSE file for details.

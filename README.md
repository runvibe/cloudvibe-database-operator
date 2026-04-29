# CloudVibe Database Operator

Kubernetes operator for declarative Aurora PostgreSQL database access provisioning.

The public API is Kubernetes-native:

- `DatabaseInstance` describes an Aurora PostgreSQL target.
- `DatabaseAccess` describes an application's database, users and permissions.

The current implementation includes the Rust/kube-rs foundation, generated CRDs,
an Axum operational HTTP server, OpenTelemetry tracing setup, initial RBAC,
Helm packaging, and LocalStack/PostgreSQL E2E scaffolding.

## Development

```sh
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo run -- export-crds > deploy/crds/database.cloudvibe.dev.yaml
```

## Local E2E Services

```sh
docker compose -f docker-compose.e2e.yaml up -d postgres localstack
scripts/e2e-bootstrap-localstack.sh
```

Run the full local E2E test:

```sh
make e2e
```

## HTTP Endpoints

- `GET /healthz`
- `GET /readyz`
- `GET /metrics`

## Installation

- [EKS installation guide](docs/eks-install.md)

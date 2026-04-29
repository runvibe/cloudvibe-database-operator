# Rust Dependencies

Versions were checked with `cargo info` before being added to `Cargo.toml`.

| Crate | Version | Note |
| --- | ---: | --- |
| `tokio` | `1.52.1` | Latest stable reported by `cargo info`. |
| `kube` | `3.1.0` | Latest stable reported by `cargo info`. |
| `k8s-openapi` | `0.27.1` | Uses Kubernetes `v1_35` feature. |
| `serde` | `1.0.228` | Latest stable reported by `cargo info`. |
| `schemars` | `1.2.1` | Latest stable reported by `cargo info`. |
| `thiserror` | `2.0.18` | Latest stable reported by `cargo info`. |
| `tracing` | `0.1.44` | Latest stable reported by `cargo info`. |
| `tracing-subscriber` | `0.3.23` | Latest stable reported by `cargo info`. |
| `tracing-opentelemetry` | `0.32.1` | Latest stable reported by `cargo info`. |
| `opentelemetry` | `0.31.0` | Latest stable reported by `cargo info`. |
| `opentelemetry-otlp` | `0.31.1` | Latest stable reported by `cargo info`. |
| `opentelemetry_sdk` | `0.31.0` | Latest stable reported by `cargo info`. |
| `axum` | `0.8.9` | Latest stable reported by `cargo info`. |
| `tower-http` | `0.6.8` | Latest stable reported by `cargo info`. |
| `sqlx` | `0.8.6` | Latest stable; `0.9.0-alpha.1` was intentionally skipped. |
| `aws-config` | `1.8.16` | Latest stable reported by `cargo info`. |
| `aws-credential-types` | `1.2.14` | Latest stable reported by `cargo info`. |
| `aws-sdk-secretsmanager` | `1.104.0` | Latest stable reported by `cargo info`. |
| `rand` | `0.10.1` | Latest stable reported by `cargo info`. |
| `regex` | `1.12.3` | Latest stable reported by `cargo info`. |
| `rustls` | `0.23.40` | Latest stable; direct dependency selects the process crypto provider. |
| `chrono` | `0.4.44` | Latest stable reported by `cargo info`. |
| `serde_json` | `1.0.149` | Latest stable reported by `cargo info`. |
| `futures` | `0.3.32` | Latest stable reported by `cargo info`. |
| `serde_yml` | `0.0.12` | Used instead of deprecated `serde_yaml`. |

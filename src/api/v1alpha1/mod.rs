use chrono::{DateTime, Utc};
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DatabaseEngine {
    AuroraPostgres,
}

#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DatabasePermission {
    Readonly,
    Readwrite,
}

#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
pub struct LocalObjectReference {
    pub name: String,
}

#[derive(Clone, CustomResource, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[kube(
    group = "database.runvibe.dev",
    version = "v1alpha1",
    kind = "DatabaseInstance",
    plural = "databaseinstances",
    namespaced,
    status = "DatabaseInstanceStatus",
    derive = "PartialEq",
    printcolumn = r#"{"name":"Engine","type":"string","jsonPath":".spec.engine"}"#,
    printcolumn = r#"{"name":"Host","type":"string","jsonPath":".spec.host"}"#,
    printcolumn = r#"{"name":"Ready","type":"string","jsonPath":".status.phase"}"#
)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseInstanceSpec {
    pub engine: DatabaseEngine,
    pub region: String,
    pub host: String,
    pub port: u16,
    pub admin_secret_arn: String,
    pub secret_prefix: String,
    #[serde(default)]
    pub allowed_namespaces: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseInstanceStatus {
    pub phase: Option<String>,
    #[serde(default)]
    pub conditions: Vec<Condition>,
}

#[derive(Clone, CustomResource, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[kube(
    group = "database.runvibe.dev",
    version = "v1alpha1",
    kind = "DatabaseAccess",
    plural = "databaseaccesses",
    namespaced,
    status = "DatabaseAccessStatus",
    derive = "PartialEq",
    printcolumn = r#"{"name":"Instance","type":"string","jsonPath":".spec.instanceRef.name"}"#,
    printcolumn = r#"{"name":"Database","type":"string","jsonPath":".spec.database"}"#,
    printcolumn = r#"{"name":"Phase","type":"string","jsonPath":".status.phase"}"#
)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseAccessSpec {
    pub instance_ref: LocalObjectReference,
    pub database: String,
    #[serde(default = "default_schemas")]
    pub schemas: Vec<String>,
    #[serde(default)]
    pub users: Vec<DatabaseUserSpec>,
}

#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseUserSpec {
    pub name: String,
    pub permissions: DatabasePermission,
    pub secret_name: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseAccessStatus {
    pub observed_generation: Option<i64>,
    pub phase: Option<String>,
    pub database: Option<String>,
    #[serde(default)]
    pub users: Vec<DatabaseAccessUserStatus>,
    #[serde(default)]
    pub conditions: Vec<Condition>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseAccessUserStatus {
    pub name: String,
    pub permissions: DatabasePermission,
    pub secret_arn: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    #[serde(rename = "type")]
    pub type_: String,
    pub status: String,
    pub reason: String,
    pub message: String,
    pub last_transition_time: DateTime<Utc>,
}

fn default_schemas() -> Vec<String> {
    vec!["public".to_string()]
}

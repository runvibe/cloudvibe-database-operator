use aws_sdk_secretsmanager::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::instrument;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminSecret {
    pub username: String,
    pub password: String,
    pub database: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSecret {
    pub engine: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub jdbc_url: String,
    pub uri: String,
}

#[derive(Clone)]
pub struct SecretsManagerStore {
    client: Client,
}

#[derive(Debug, Error)]
pub enum SecretStoreError {
    #[error("secret `{0}` has no string value")]
    MissingString(String),
    #[error("failed to parse secret json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("aws secrets manager error: {0}")]
    Aws(String),
}

impl SecretsManagerStore {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    #[instrument(skip(self), fields(secret_arn))]
    pub async fn get_admin_secret(
        &self,
        secret_arn: &str,
    ) -> Result<AdminSecret, SecretStoreError> {
        let output = self
            .client
            .get_secret_value()
            .secret_id(secret_arn)
            .send()
            .await
            .map_err(|err| SecretStoreError::Aws(err.to_string()))?;
        let value = output
            .secret_string()
            .ok_or_else(|| SecretStoreError::MissingString(secret_arn.to_string()))?;
        Ok(serde_json::from_str(value)?)
    }

    #[instrument(skip(self, secret), fields(secret_name))]
    pub async fn put_app_secret(
        &self,
        secret_name: &str,
        secret: &AppSecret,
    ) -> Result<String, SecretStoreError> {
        let value = serde_json::to_string(secret)?;
        match self
            .client
            .create_secret()
            .name(secret_name)
            .secret_string(value.clone())
            .send()
            .await
        {
            Ok(output) => Ok(output.arn().unwrap_or(secret_name).to_string()),
            Err(_) => {
                self.client
                    .put_secret_value()
                    .secret_id(secret_name)
                    .secret_string(value)
                    .send()
                    .await
                    .map_err(|err| SecretStoreError::Aws(err.to_string()))?;
                Ok(secret_name.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_app_secret_without_renaming_password() {
        let secret = AppSecret {
            engine: "aurora-postgres".to_string(),
            host: "db.example".to_string(),
            port: 5432,
            database: "orders".to_string(),
            username: "orders_api".to_string(),
            password: "secret".to_string(),
            jdbc_url: "jdbc:postgresql://db.example:5432/orders".to_string(),
            uri: "postgresql://orders_api:secret@db.example:5432/orders".to_string(),
        };
        let json = serde_json::to_string(&secret).unwrap();
        assert!(json.contains("\"password\""));
        assert!(!json.contains("secretArn"));
    }

    #[test]
    fn parses_admin_secret() {
        let json = r#"{"username":"postgres","password":"secret","database":"postgres"}"#;
        let secret: AdminSecret = serde_json::from_str(json).unwrap();
        assert_eq!(secret.username, "postgres");
        assert_eq!(secret.database, "postgres");
    }
}

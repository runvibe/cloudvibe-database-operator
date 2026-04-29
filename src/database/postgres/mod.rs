use crate::api::v1alpha1::DatabasePermission;
use crate::naming::{NamingError, quote_identifier, validate_identifier};
use sqlx::{Executor, PgPool};
use thiserror::Error;
use tracing::instrument;

#[derive(Clone, Debug)]
pub struct ProvisionRequest {
    pub database: String,
    pub schemas: Vec<String>,
    pub users: Vec<ProvisionUser>,
}

#[derive(Clone, Debug)]
pub struct ProvisionUser {
    pub name: String,
    pub password: String,
    pub permissions: DatabasePermission,
}

#[derive(Clone)]
pub struct PostgresProvisioner {
    admin_pool: PgPool,
}

#[derive(Debug, Error)]
pub enum ProvisionError {
    #[error(transparent)]
    Naming(#[from] NamingError),
    #[error("postgres error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl PostgresProvisioner {
    pub fn new(admin_pool: PgPool) -> Self {
        Self { admin_pool }
    }

    #[instrument(skip(self, request), fields(database = request.database))]
    pub async fn provision(&self, request: &ProvisionRequest) -> Result<(), ProvisionError> {
        validate_identifier(&request.database)?;
        for schema in &request.schemas {
            validate_identifier(schema)?;
        }
        for user in &request.users {
            validate_identifier(&user.name)?;
        }
        self.create_roles(request).await?;
        Ok(())
    }

    async fn create_roles(&self, request: &ProvisionRequest) -> Result<(), ProvisionError> {
        for user in &request.users {
            let role = quote_identifier(&user.name)?;
            let password = quote_literal(&user.password);
            let sql = format!("CREATE ROLE {role} LOGIN PASSWORD {password}");
            match self.admin_pool.execute(sql.as_str()).await {
                Ok(_) => {}
                Err(sqlx::Error::Database(err)) if err.code().as_deref() == Some("42710") => {}
                Err(err) => return Err(err.into()),
            }
        }
        Ok(())
    }
}

pub fn app_secret_uri(
    host: &str,
    port: u16,
    database: &str,
    username: &str,
    password: &str,
) -> String {
    format!("postgresql://{username}:{password}@{host}:{port}/{database}")
}

pub fn jdbc_url(host: &str, port: u16, database: &str) -> String {
    format!("jdbc:postgresql://{host}:{port}/{database}")
}

fn quote_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes_password_literals() {
        assert_eq!(quote_literal("abc"), "'abc'");
        assert_eq!(quote_literal("a'b"), "'a''b'");
    }
}

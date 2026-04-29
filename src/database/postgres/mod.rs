use crate::api::v1alpha1::DatabasePermission;
use crate::naming::{NamingError, quote_identifier, validate_identifier};
use sqlx::{
    Executor, PgPool,
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
};
use std::time::Duration;
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

#[derive(Clone, Debug)]
pub struct PostgresConnectionConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub ssl_mode: PgSslMode,
}

#[derive(Clone)]
pub struct PostgresProvisioner {
    admin_pool: PgPool,
    connection: Option<PostgresConnectionConfig>,
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
        Self {
            admin_pool,
            connection: None,
        }
    }

    pub async fn connect(connection: PostgresConnectionConfig) -> Result<Self, ProvisionError> {
        let pool = connect_pool(&connection).await?;
        Ok(Self {
            admin_pool: pool,
            connection: Some(connection),
        })
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
        self.create_database(&request.database).await?;
        let target_pool = self.connect_target(&request.database).await?;
        self.create_schemas(&target_pool, request).await?;
        self.create_roles(request).await?;
        self.apply_grants(&target_pool, request).await?;
        Ok(())
    }

    async fn create_database(&self, database: &str) -> Result<(), ProvisionError> {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)")
                .bind(database)
                .fetch_one(&self.admin_pool)
                .await?;
        if exists {
            return Ok(());
        }
        let database = quote_identifier(database)?;
        self.admin_pool
            .execute(format!("CREATE DATABASE {database}").as_str())
            .await?;
        Ok(())
    }

    async fn connect_target(&self, database: &str) -> Result<PgPool, ProvisionError> {
        let Some(mut connection) = self.connection.clone() else {
            return Ok(self.admin_pool.clone());
        };
        connection.database = database.to_string();
        connect_pool(&connection).await
    }

    async fn create_schemas(
        &self,
        target_pool: &PgPool,
        request: &ProvisionRequest,
    ) -> Result<(), ProvisionError> {
        for schema in &request.schemas {
            let schema = quote_identifier(schema)?;
            target_pool
                .execute(format!("CREATE SCHEMA IF NOT EXISTS {schema}").as_str())
                .await?;
        }
        Ok(())
    }

    async fn create_roles(&self, request: &ProvisionRequest) -> Result<(), ProvisionError> {
        for user in &request.users {
            let role = quote_identifier(&user.name)?;
            let password = quote_literal(&user.password);
            let sql = format!("CREATE ROLE {role} LOGIN PASSWORD {password}");
            match self.admin_pool.execute(sql.as_str()).await {
                Ok(_) => {}
                Err(sqlx::Error::Database(err)) if err.code().as_deref() == Some("42710") => {
                    self.admin_pool
                        .execute(format!("ALTER ROLE {role} PASSWORD {password}").as_str())
                        .await?;
                }
                Err(err) => return Err(err.into()),
            }
        }
        Ok(())
    }

    async fn apply_grants(
        &self,
        target_pool: &PgPool,
        request: &ProvisionRequest,
    ) -> Result<(), ProvisionError> {
        let database = quote_identifier(&request.database)?;
        for user in &request.users {
            let role = quote_identifier(&user.name)?;
            self.admin_pool
                .execute(format!("GRANT CONNECT ON DATABASE {database} TO {role}").as_str())
                .await?;

            for schema in &request.schemas {
                let schema = quote_identifier(schema)?;
                match user.permissions {
                    DatabasePermission::Readonly => {
                        target_pool
                            .execute(
                                format!(
                                    "GRANT USAGE ON SCHEMA {schema} TO {role}; \
                                     GRANT SELECT ON ALL TABLES IN SCHEMA {schema} TO {role}; \
                                     ALTER DEFAULT PRIVILEGES IN SCHEMA {schema} GRANT SELECT ON TABLES TO {role}"
                                )
                                .as_str(),
                            )
                            .await?;
                    }
                    DatabasePermission::Readwrite => {
                        target_pool
                            .execute(
                                format!(
                                    "GRANT USAGE, CREATE ON SCHEMA {schema} TO {role}; \
                                     GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA {schema} TO {role}; \
                                     GRANT USAGE, SELECT, UPDATE ON ALL SEQUENCES IN SCHEMA {schema} TO {role}; \
                                     ALTER DEFAULT PRIVILEGES IN SCHEMA {schema} GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO {role}; \
                                     ALTER DEFAULT PRIVILEGES IN SCHEMA {schema} GRANT USAGE, SELECT, UPDATE ON SEQUENCES TO {role}"
                                )
                                .as_str(),
                            )
                            .await?;
                    }
                }
            }
        }
        Ok(())
    }
}

async fn connect_pool(connection: &PostgresConnectionConfig) -> Result<PgPool, ProvisionError> {
    let options = PgConnectOptions::new()
        .host(&connection.host)
        .port(connection.port)
        .database(&connection.database)
        .username(&connection.username)
        .password(&connection.password)
        .ssl_mode(connection.ssl_mode);

    Ok(PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .connect_with(options)
        .await?)
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

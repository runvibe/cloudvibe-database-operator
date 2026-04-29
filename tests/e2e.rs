use aws_config::{BehaviorVersion, Region};
use aws_credential_types::Credentials;
use aws_sdk_secretsmanager::Client;
use cloudvibe_database_operator::{
    api::v1alpha1::DatabasePermission,
    aws::secretsmanager::{AppSecret, SecretsManagerStore},
    database::postgres::{
        PostgresConnectionConfig, PostgresProvisioner, ProvisionRequest, ProvisionUser,
        app_secret_uri, jdbc_url,
    },
    password,
};
use sqlx::postgres::PgSslMode;
use sqlx::{Executor, PgPool, Row};

#[tokio::test]
#[ignore = "requires docker compose services from docker-compose.e2e.yaml"]
async fn localstack_and_postgres_e2e() {
    let database_url = env_or(
        "E2E_DATABASE_URL",
        "postgres://postgres:postgres@localhost:5432/postgres",
    );
    let aws_endpoint = env_or("AWS_ENDPOINT_URL", "http://localhost:4566");
    let region = env_or("AWS_REGION", "us-east-1");
    let secret_name = env_or("E2E_APP_SECRET_NAME", "rds/prod-main/e2e/orders_api_rw");
    let pg_host = env_or("E2E_POSTGRES_HOST", "localhost");
    let pg_port: u16 = env_or("E2E_POSTGRES_PORT", "5432")
        .parse()
        .expect("valid postgres port");
    let admin_user = env_or("E2E_POSTGRES_USER", "postgres");
    let admin_password = env_or("E2E_POSTGRES_PASSWORD", "postgres");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("connect postgres");
    let suffix = std::process::id();
    let database_name = format!("e2e_db_{suffix}");
    let rw_role_name = format!("e2e_rw_{suffix}");
    let ro_role_name = format!("e2e_ro_{suffix}");
    let rw_password = password::generate(32);
    let ro_password = password::generate(32);

    let provisioner = PostgresProvisioner::connect(PostgresConnectionConfig {
        host: pg_host.clone(),
        port: pg_port,
        database: "postgres".to_string(),
        username: admin_user.clone(),
        password: admin_password.clone(),
        ssl_mode: PgSslMode::Prefer,
    })
    .await
    .expect("connect provisioner");
    provisioner
        .provision(&ProvisionRequest {
            database: database_name.clone(),
            schemas: vec!["public".to_string()],
            users: vec![
                ProvisionUser {
                    name: rw_role_name.clone(),
                    password: rw_password.clone(),
                    permissions: DatabasePermission::Readwrite,
                },
                ProvisionUser {
                    name: ro_role_name.clone(),
                    password: ro_password.clone(),
                    permissions: DatabasePermission::Readonly,
                },
            ],
        })
        .await
        .expect("provision postgres database and roles");

    let database_exists: bool =
        sqlx::query("SELECT EXISTS (SELECT 1 FROM pg_database WHERE datname = $1)")
            .bind(&database_name)
            .fetch_one(&pool)
            .await
            .expect("query database")
            .get(0);
    assert!(database_exists, "provisioned database must exist");

    let rw_exists: bool = sqlx::query("SELECT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = $1)")
        .bind(&rw_role_name)
        .fetch_one(&pool)
        .await
        .expect("query rw role")
        .get(0);
    assert!(rw_exists, "provisioned readwrite role must exist");

    let target_admin = postgres_url(
        &pg_host,
        pg_port,
        &database_name,
        &admin_user,
        &admin_password,
    );
    let target_pool = PgPool::connect(&target_admin)
        .await
        .expect("connect target database as admin");
    target_pool
        .execute("CREATE TABLE public.e2e_items (id SERIAL PRIMARY KEY, name TEXT NOT NULL)")
        .await
        .expect("create table with default privileges");
    target_pool
        .execute("INSERT INTO public.e2e_items (name) VALUES ('seed')")
        .await
        .expect("insert seed row");

    let ro_pool = PgPool::connect(&postgres_url(
        &pg_host,
        pg_port,
        &database_name,
        &ro_role_name,
        &ro_password,
    ))
    .await
    .expect("connect readonly user");
    let row_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM public.e2e_items")
        .fetch_one(&ro_pool)
        .await
        .expect("readonly can select");
    assert_eq!(row_count, 1);
    let readonly_insert = ro_pool
        .execute("INSERT INTO public.e2e_items (name) VALUES ('blocked')")
        .await;
    assert!(readonly_insert.is_err(), "readonly must not insert");

    let rw_pool = PgPool::connect(&postgres_url(
        &pg_host,
        pg_port,
        &database_name,
        &rw_role_name,
        &rw_password,
    ))
    .await
    .expect("connect readwrite user");
    rw_pool
        .execute("INSERT INTO public.e2e_items (name) VALUES ('allowed')")
        .await
        .expect("readwrite can insert");

    let client = secrets_client(&aws_endpoint, &region).await;
    let store = SecretsManagerStore::new(client.clone());
    let app_secret = AppSecret {
        engine: "aurora-postgres".to_string(),
        host: pg_host.clone(),
        port: pg_port,
        database: database_name.clone(),
        username: rw_role_name.clone(),
        password: rw_password.clone(),
        jdbc_url: jdbc_url(&pg_host, pg_port, &database_name),
        uri: app_secret_uri(
            &pg_host,
            pg_port,
            &database_name,
            &rw_role_name,
            &rw_password,
        ),
    };

    store
        .put_app_secret(&secret_name, &app_secret)
        .await
        .expect("put app secret");

    let output = client
        .get_secret_value()
        .secret_id(&secret_name)
        .send()
        .await
        .expect("read app secret");
    let secret_string = output.secret_string().expect("secret string");
    assert!(secret_string.contains(&rw_role_name));
    assert!(!secret_string.contains("secretArn"));

    ro_pool.close().await;
    rw_pool.close().await;
    target_pool.close().await;
    let _ = pool
        .execute(
            format!(
                "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}'",
                database_name
            )
            .as_str(),
        )
        .await;
    let _ = pool
        .execute(format!(r#"DROP DATABASE IF EXISTS "{database_name}""#).as_str())
        .await;
    let _ = pool
        .execute(format!(r#"DROP ROLE IF EXISTS "{rw_role_name}""#).as_str())
        .await;
    let _ = pool
        .execute(format!(r#"DROP ROLE IF EXISTS "{ro_role_name}""#).as_str())
        .await;
}

async fn secrets_client(endpoint: &str, region: &str) -> Client {
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new(region.to_string()))
        .endpoint_url(endpoint)
        .credentials_provider(Credentials::new("test", "test", None, None, "e2e"))
        .load()
        .await;
    Client::new(&config)
}

fn env_or(name: &str, fallback: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| fallback.to_string())
}

fn postgres_url(host: &str, port: u16, database: &str, username: &str, password: &str) -> String {
    format!("postgres://{username}:{password}@{host}:{port}/{database}")
}

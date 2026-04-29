use aws_config::{BehaviorVersion, Region};
use aws_credential_types::Credentials;
use aws_sdk_secretsmanager::Client;
use cloudvibe_database_operator::{
    api::v1alpha1::DatabasePermission,
    aws::secretsmanager::{AppSecret, SecretsManagerStore},
    database::postgres::{
        PostgresProvisioner, ProvisionRequest, ProvisionUser, app_secret_uri, jdbc_url,
    },
    password,
};
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

    let pool = PgPool::connect(&database_url)
        .await
        .expect("connect postgres");
    let role_name = format!("e2e_user_{}", std::process::id());
    let password = password::generate(32);

    let provisioner = PostgresProvisioner::new(pool.clone());
    provisioner
        .provision(&ProvisionRequest {
            database: "postgres".to_string(),
            schemas: vec!["public".to_string()],
            users: vec![ProvisionUser {
                name: role_name.clone(),
                password: password.clone(),
                permissions: DatabasePermission::Readwrite,
            }],
        })
        .await
        .expect("provision postgres role");

    let exists: bool = sqlx::query("SELECT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = $1)")
        .bind(&role_name)
        .fetch_one(&pool)
        .await
        .expect("query role")
        .get(0);
    assert!(exists, "provisioned role must exist");

    let client = secrets_client(&aws_endpoint, &region).await;
    let store = SecretsManagerStore::new(client.clone());
    let app_secret = AppSecret {
        engine: "aurora-postgres".to_string(),
        host: "localhost".to_string(),
        port: 5432,
        database: "postgres".to_string(),
        username: role_name.clone(),
        password: password.clone(),
        jdbc_url: jdbc_url("localhost", 5432, "postgres"),
        uri: app_secret_uri("localhost", 5432, "postgres", &role_name, &password),
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
    assert!(secret_string.contains(&role_name));
    assert!(!secret_string.contains("secretArn"));

    let drop_role = format!(r#"DROP ROLE IF EXISTS "{}""#, role_name);
    let _ = pool.execute(drop_role.as_str()).await;
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

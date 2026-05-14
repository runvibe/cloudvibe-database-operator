use crate::api::v1alpha1::{
    DatabaseAccess, DatabaseAccessStatus, DatabaseAccessUserStatus, DatabaseEngine,
    DatabaseInstance,
};
use crate::aws::secretsmanager::{AppSecret, SecretStoreError, SecretsManagerStore};
use crate::database::postgres::{
    PostgresConnectionConfig, PostgresProvisioner, ProvisionError, ProvisionRequest, ProvisionUser,
    app_secret_uri, jdbc_url,
};
use crate::naming::{self, NamingError};
use crate::password;
use aws_config::BehaviorVersion;
use aws_sdk_secretsmanager::config::Region;
use futures::StreamExt;
use kube::{
    Api, Client, ResourceExt,
    api::{Patch, PatchParams},
    runtime::{Controller, controller::Action, watcher},
};
use sqlx::postgres::PgSslMode;
use std::{sync::Arc, time::Duration};
use thiserror::Error;
use tracing::{error, info, instrument};

mod status;

#[derive(Clone)]
pub struct Context {
    client: Client,
}

#[derive(Debug, Error)]
pub enum ReconcileError {
    #[error("kubernetes api error: {0}")]
    Kube(#[from] kube::Error),
    #[error(transparent)]
    SecretStore(#[from] SecretStoreError),
    #[error(transparent)]
    Provision(#[from] ProvisionError),
    #[error(transparent)]
    Naming(#[from] NamingError),
}

pub async fn run(client: Client) {
    let accesses = Api::<DatabaseAccess>::all(client.clone());
    let instances = Api::<DatabaseInstance>::all(client.clone());
    let context = Arc::new(Context { client });

    Controller::new(accesses, watcher::Config::default())
        .owns(instances, watcher::Config::default())
        .run(reconcile, error_policy, context)
        .for_each(|result| async move {
            if let Err(err) = result {
                error!(error = %err, "reconcile failed");
            }
        })
        .await;
}

#[instrument(skip(access, ctx), fields(name = access.name_any(), namespace = access.namespace().unwrap_or_default()))]
async fn reconcile(
    access: Arc<DatabaseAccess>,
    ctx: Arc<Context>,
) -> Result<Action, ReconcileError> {
    let namespace = access.namespace().unwrap_or_default();
    let instances: Api<DatabaseInstance> = Api::namespaced(ctx.client.clone(), &namespace);
    let instance = instances.get(&access.spec.instance_ref.name).await;

    let status = match instance {
        Ok(instance) if namespace_allowed(&instance, &namespace) => {
            match provision_access(&access, &instance, &namespace).await {
                Ok(status) => status,
                Err(err) => {
                    error!(error = %err, "database access provisioning failed");
                    status::provisioning_error_status(&access, &err)
                }
            }
        }
        Ok(_) => status::error_status(
            &access,
            "NamespaceDenied",
            "namespace is not allowed for instance",
        ),
        Err(kube::Error::Api(err)) if err.code == 404 => status::error_status(
            &access,
            "InstanceNotFound",
            "referenced DatabaseInstance was not found",
        ),
        Err(err) => return Err(err.into()),
    };

    if status::needs_patch(access.status.as_ref(), &status) {
        let accesses: Api<DatabaseAccess> = Api::namespaced(ctx.client.clone(), &namespace);
        accesses
            .patch_status(
                &access.name_any(),
                &PatchParams::apply("cloudvibe-database-operator"),
                &Patch::Merge(serde_json::json!({ "status": status })),
            )
            .await?;
    }

    Ok(Action::requeue(Duration::from_secs(300)))
}

async fn provision_access(
    access: &DatabaseAccess,
    instance: &DatabaseInstance,
    namespace: &str,
) -> Result<DatabaseAccessStatus, ReconcileError> {
    info!(
        database = access.spec.database,
        instance = access.spec.instance_ref.name,
        "provisioning database access"
    );

    let store = secret_store(&instance.spec.region).await;
    let admin_secret = store
        .get_admin_secret(&instance.spec.admin_secret_arn)
        .await?;
    let mut provision_users = Vec::with_capacity(access.spec.users.len());
    let mut status_users = Vec::with_capacity(access.spec.users.len());
    let mut pending_secrets = Vec::new();

    for user in &access.spec.users {
        let secret_name = match &user.secret_name {
            Some(name) => {
                naming::ensure_secret_prefix(&instance.spec.secret_prefix, name)?;
                name.clone()
            }
            None => naming::default_secret_name(
                &instance.spec.secret_prefix,
                namespace,
                &access.spec.database,
                &user.name,
            )?,
        };

        let existing_secret = store.get_app_secret(&secret_name).await?;
        let password = existing_secret
            .as_ref()
            .map(|secret| secret.password.clone())
            .unwrap_or_else(|| password::generate(40));

        provision_users.push(ProvisionUser {
            name: user.name.clone(),
            password: password.clone(),
            permissions: user.permissions.clone(),
        });

        if existing_secret.is_none() {
            let uri = app_secret_uri(
                &instance.spec.host,
                instance.spec.port,
                &access.spec.database,
                &user.name,
                &password,
            );
            let app_secret = AppSecret {
                engine: engine_name(&instance.spec.engine).to_string(),
                host: instance.spec.host.clone(),
                port: instance.spec.port,
                database: access.spec.database.clone(),
                username: user.name.clone(),
                password,
                jdbc_url: jdbc_url(
                    &instance.spec.host,
                    instance.spec.port,
                    &access.spec.database,
                ),
                uri,
            };
            pending_secrets.push((status_users.len(), secret_name.clone(), app_secret));
        }

        status_users.push(DatabaseAccessUserStatus {
            name: user.name.clone(),
            permissions: user.permissions.clone(),
            secret_arn: secret_name,
        });
    }

    let provisioner = PostgresProvisioner::connect(PostgresConnectionConfig {
        host: instance.spec.host.clone(),
        port: instance.spec.port,
        database: admin_secret.database,
        username: admin_secret.username,
        password: admin_secret.password,
        ssl_mode: PgSslMode::Prefer,
    })
    .await?;
    provisioner
        .provision(&ProvisionRequest {
            database: access.spec.database.clone(),
            schemas: access.spec.schemas.clone(),
            users: provision_users,
        })
        .await?;

    for (index, secret_name, app_secret) in pending_secrets {
        let secret_arn = store.put_app_secret(&secret_name, &app_secret).await?;
        if let Some(status_user) = status_users.get_mut(index) {
            status_user.secret_arn = secret_arn;
        }
    }

    Ok(status::ready_status(access, status_users))
}

async fn secret_store(region: &str) -> SecretsManagerStore {
    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new(region.to_string()))
        .load()
        .await;
    SecretsManagerStore::new(aws_sdk_secretsmanager::Client::new(&config))
}

fn namespace_allowed(instance: &DatabaseInstance, namespace: &str) -> bool {
    instance.spec.allowed_namespaces.is_empty()
        || instance
            .spec
            .allowed_namespaces
            .iter()
            .any(|allowed| allowed == namespace)
}

fn error_policy(_object: Arc<DatabaseAccess>, _err: &ReconcileError, _ctx: Arc<Context>) -> Action {
    Action::requeue(Duration::from_secs(60))
}

fn engine_name(engine: &DatabaseEngine) -> &'static str {
    match engine {
        DatabaseEngine::AuroraPostgres => "aurora-postgres",
    }
}

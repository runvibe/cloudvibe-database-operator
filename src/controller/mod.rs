use crate::api::v1alpha1::{Condition, DatabaseAccess, DatabaseAccessStatus, DatabaseInstance};
use chrono::Utc;
use futures::StreamExt;
use kube::{
    Api, Client, ResourceExt,
    api::{Patch, PatchParams},
    runtime::{Controller, controller::Action, watcher},
};
use std::{sync::Arc, time::Duration};
use thiserror::Error;
use tracing::{error, instrument};

#[derive(Clone)]
pub struct Context {
    client: Client,
}

#[derive(Debug, Error)]
pub enum ReconcileError {
    #[error("kubernetes api error: {0}")]
    Kube(#[from] kube::Error),
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
        Ok(instance) if namespace_allowed(&instance, &namespace) => ready_status(&access),
        Ok(_) => error_status(
            &access,
            "NamespaceDenied",
            "namespace is not allowed for instance",
        ),
        Err(kube::Error::Api(err)) if err.code == 404 => error_status(
            &access,
            "InstanceNotFound",
            "referenced DatabaseInstance was not found",
        ),
        Err(err) => return Err(err.into()),
    };

    let accesses: Api<DatabaseAccess> = Api::namespaced(ctx.client.clone(), &namespace);
    accesses
        .patch_status(
            &access.name_any(),
            &PatchParams::apply("cloudvibe-database-operator"),
            &Patch::Merge(serde_json::json!({ "status": status })),
        )
        .await?;
    Ok(Action::requeue(Duration::from_secs(300)))
}

fn namespace_allowed(instance: &DatabaseInstance, namespace: &str) -> bool {
    instance.spec.allowed_namespaces.is_empty()
        || instance
            .spec
            .allowed_namespaces
            .iter()
            .any(|allowed| allowed == namespace)
}

fn ready_status(access: &DatabaseAccess) -> DatabaseAccessStatus {
    DatabaseAccessStatus {
        observed_generation: access.metadata.generation,
        phase: Some("Ready".to_string()),
        database: Some(access.spec.database.clone()),
        users: Vec::new(),
        conditions: vec![condition(
            "Ready",
            "True",
            "Validated",
            "database access is valid",
        )],
    }
}

fn error_status(access: &DatabaseAccess, reason: &str, message: &str) -> DatabaseAccessStatus {
    DatabaseAccessStatus {
        observed_generation: access.metadata.generation,
        phase: Some("Error".to_string()),
        database: Some(access.spec.database.clone()),
        users: Vec::new(),
        conditions: vec![condition("Ready", "False", reason, message)],
    }
}

fn condition(type_: &str, status: &str, reason: &str, message: &str) -> Condition {
    Condition {
        type_: type_.to_string(),
        status: status.to_string(),
        reason: reason.to_string(),
        message: message.to_string(),
        last_transition_time: Utc::now(),
    }
}

fn error_policy(_object: Arc<DatabaseAccess>, _err: &ReconcileError, _ctx: Arc<Context>) -> Action {
    Action::requeue(Duration::from_secs(60))
}

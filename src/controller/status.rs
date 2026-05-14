use super::ReconcileError;
use crate::api::v1alpha1::{
    Condition, DatabaseAccess, DatabaseAccessStatus, DatabaseAccessUserStatus,
};
use chrono::Utc;

pub(super) fn needs_patch(
    current: Option<&DatabaseAccessStatus>,
    desired: &DatabaseAccessStatus,
) -> bool {
    match current {
        Some(current) => !status_matches(current, desired),
        None => true,
    }
}

pub(super) fn ready_status(
    access: &DatabaseAccess,
    users: Vec<DatabaseAccessUserStatus>,
) -> DatabaseAccessStatus {
    DatabaseAccessStatus {
        observed_generation: access.metadata.generation,
        phase: Some("Ready".to_string()),
        database: Some(access.spec.database.clone()),
        users,
        conditions: vec![condition(
            "Ready",
            "True",
            "Provisioned",
            "database access is ready",
        )],
    }
}

pub(super) fn error_status(
    access: &DatabaseAccess,
    reason: &str,
    message: &str,
) -> DatabaseAccessStatus {
    DatabaseAccessStatus {
        observed_generation: access.metadata.generation,
        phase: Some("Error".to_string()),
        database: Some(access.spec.database.clone()),
        users: Vec::new(),
        conditions: vec![condition("Ready", "False", reason, message)],
    }
}

pub(super) fn provisioning_error_status(
    access: &DatabaseAccess,
    err: &ReconcileError,
) -> DatabaseAccessStatus {
    let (reason, message) = match err {
        ReconcileError::SecretStore(_) => (
            "SecretStoreError",
            "failed to read or write AWS Secrets Manager secret",
        ),
        ReconcileError::Provision(_) => (
            "ProvisioningError",
            "failed to provision database resources",
        ),
        ReconcileError::Naming(_) => ("InvalidName", "database access contains an invalid name"),
        ReconcileError::Kube(_) => ("KubernetesError", "failed to call the Kubernetes API"),
    };
    error_status(access, reason, message)
}

fn status_matches(current: &DatabaseAccessStatus, desired: &DatabaseAccessStatus) -> bool {
    current.observed_generation == desired.observed_generation
        && current.phase == desired.phase
        && current.database == desired.database
        && current.users == desired.users
        && conditions_match(&current.conditions, &desired.conditions)
}

fn conditions_match(current: &[Condition], desired: &[Condition]) -> bool {
    current.len() == desired.len()
        && current
            .iter()
            .zip(desired)
            .all(|(current, desired)| condition_matches(current, desired))
}

fn condition_matches(current: &Condition, desired: &Condition) -> bool {
    current.type_ == desired.type_
        && current.status == desired.status
        && current.reason == desired.reason
        && current.message == desired.message
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

#[cfg(test)]
mod tests {
    use crate::api::v1alpha1::{
        Condition, DatabaseAccessStatus, DatabaseAccessUserStatus, DatabasePermission,
    };
    use chrono::{TimeZone, Utc};

    #[test]
    fn does_not_patch_when_only_condition_transition_time_changes() {
        let current = ready_status_at(Utc.with_ymd_and_hms(2026, 5, 13, 10, 0, 0).unwrap());
        let desired = ready_status_at(Utc.with_ymd_and_hms(2026, 5, 13, 10, 5, 0).unwrap());

        assert!(!super::needs_patch(Some(&current), &desired));
    }

    #[test]
    fn patches_when_condition_reason_changes() {
        let current = ready_status_at(Utc.with_ymd_and_hms(2026, 5, 13, 10, 0, 0).unwrap());
        let mut desired = ready_status_at(Utc.with_ymd_and_hms(2026, 5, 13, 10, 5, 0).unwrap());
        desired.conditions[0].reason = "ProvisioningError".to_string();

        assert!(super::needs_patch(Some(&current), &desired));
    }

    fn ready_status_at(last_transition_time: chrono::DateTime<Utc>) -> DatabaseAccessStatus {
        DatabaseAccessStatus {
            observed_generation: Some(1),
            phase: Some("Ready".to_string()),
            database: Some("auth".to_string()),
            users: vec![DatabaseAccessUserStatus {
                name: "auth_rw".to_string(),
                permissions: DatabasePermission::Readwrite,
                secret_arn: "autob-k8s/database-operator/auth/auth/auth_rw".to_string(),
            }],
            conditions: vec![Condition {
                type_: "Ready".to_string(),
                status: "True".to_string(),
                reason: "Provisioned".to_string(),
                message: "database access is ready".to_string(),
                last_transition_time,
            }],
        }
    }
}

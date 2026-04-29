# EKS Installation Guide

This guide installs CloudVibe Database Operator on Amazon EKS and connects it to
Aurora PostgreSQL and AWS Secrets Manager.

## Prerequisites

- An EKS cluster with `kubectl` access.
- Helm 3 installed locally or in CI.
- An Aurora PostgreSQL cluster reachable from the EKS worker nodes.
- AWS Secrets Manager enabled in the same account or in an allowed account.
- IRSA or EKS Pod Identity configured for the cluster.
- Optional, but recommended: External Secrets Operator installed in the cluster.

The operator image is published to GHCR:

```sh
ghcr.io/cloudvibedev/cloudvibe-database-operator:latest
```

For production, prefer a commit SHA tag instead of `latest`.

## 1. Create The Aurora Admin Secret

Create one AWS Secrets Manager secret with the administrative database
credential. The operator uses this secret to create databases, roles and grants.

Expected JSON:

```json
{
  "username": "postgres",
  "password": "replace-me",
  "database": "postgres"
}
```

Example:

```sh
aws secretsmanager create-secret \
  --name rds/prod-main/admin \
  --region us-east-1 \
  --secret-string '{
    "username": "postgres",
    "password": "replace-me",
    "database": "postgres"
  }'
```

Keep the returned ARN. It is used in `DatabaseInstance.spec.adminSecretArn`.

## 2. Create The Operator IAM Policy

Start with the smallest practical Secrets Manager permissions. Scope the
resources to the admin secret and the application secret prefix used by the
operator.

Example policy:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "ReadAdminSecret",
      "Effect": "Allow",
      "Action": [
        "secretsmanager:GetSecretValue",
        "secretsmanager:DescribeSecret"
      ],
      "Resource": "arn:aws:secretsmanager:us-east-1:123456789012:secret:rds/prod-main/admin-*"
    },
    {
      "Sid": "ManageApplicationSecrets",
      "Effect": "Allow",
      "Action": [
        "secretsmanager:GetSecretValue",
        "secretsmanager:DescribeSecret",
        "secretsmanager:CreateSecret",
        "secretsmanager:PutSecretValue",
        "secretsmanager:TagResource"
      ],
      "Resource": "arn:aws:secretsmanager:us-east-1:123456789012:secret:rds/prod-main/*"
    }
  ]
}
```

Create the policy:

```sh
aws iam create-policy \
  --policy-name CloudVibeDatabaseOperatorSecretsManager \
  --policy-document file://cloudvibe-database-operator-policy.json
```

## 3. Create The Pod IAM Role

### Option A: IRSA

Create an IAM role trusted by the EKS OIDC provider and attach the policy from
the previous step.

Trust policy shape:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Principal": {
        "Federated": "arn:aws:iam::123456789012:oidc-provider/oidc.eks.us-east-1.amazonaws.com/id/EXAMPLED539D4633E53DE1B71EXAMPLE"
      },
      "Action": "sts:AssumeRoleWithWebIdentity",
      "Condition": {
        "StringEquals": {
          "oidc.eks.us-east-1.amazonaws.com/id/EXAMPLED539D4633E53DE1B71EXAMPLE:sub": "system:serviceaccount:cloudvibe-system:cloudvibe-database-operator",
          "oidc.eks.us-east-1.amazonaws.com/id/EXAMPLED539D4633E53DE1B71EXAMPLE:aud": "sts.amazonaws.com"
        }
      }
    }
  ]
}
```

The resulting role ARN is used as a Helm value:

```text
arn:aws:iam::123456789012:role/cloudvibe-database-operator
```

### Option B: EKS Pod Identity

If the cluster uses EKS Pod Identity, create the IAM role, attach the same
policy and associate it with the operator service account:

```sh
aws eks create-pod-identity-association \
  --cluster-name prod-eks \
  --namespace cloudvibe-system \
  --service-account cloudvibe-database-operator \
  --role-arn arn:aws:iam::123456789012:role/cloudvibe-database-operator
```

When using Pod Identity, the Helm service account annotation is not required.

## 4. Install The CRDs

Install the CRDs before installing the Helm chart:

```sh
kubectl apply -f deploy/crds/database.cloudvibe.dev.yaml
```

Verify:

```sh
kubectl get crd | grep database.cloudvibe.dev
```

## 5. Install The Operator With Helm

Create the namespace:

```sh
kubectl create namespace cloudvibe-system
```

Install with IRSA:

```sh
helm upgrade --install cloudvibe-database-operator \
  charts/cloudvibe-database-operator \
  --namespace cloudvibe-system \
  --set image.repository=ghcr.io/cloudvibedev/cloudvibe-database-operator \
  --set image.tag=latest \
  --set serviceAccount.annotations."eks\.amazonaws\.com/role-arn"=arn:aws:iam::123456789012:role/cloudvibe-database-operator
```

Install with EKS Pod Identity:

```sh
helm upgrade --install cloudvibe-database-operator \
  charts/cloudvibe-database-operator \
  --namespace cloudvibe-system \
  --set image.repository=ghcr.io/cloudvibedev/cloudvibe-database-operator \
  --set image.tag=latest
```

Check rollout:

```sh
kubectl -n cloudvibe-system rollout status deployment/cloudvibe-database-operator
kubectl -n cloudvibe-system logs deploy/cloudvibe-database-operator
```

## 6. Create A DatabaseInstance

`DatabaseInstance` represents an Aurora PostgreSQL cluster that applications can
use.

```yaml
apiVersion: database.cloudvibe.dev/v1alpha1
kind: DatabaseInstance
metadata:
  name: prod-main-postgres
  namespace: orders
spec:
  engine: aurora-postgres
  region: us-east-1
  host: prod-main.cluster-xxxxxxxx.us-east-1.rds.amazonaws.com
  port: 5432
  adminSecretArn: arn:aws:secretsmanager:us-east-1:123456789012:secret:rds/prod-main/admin-AbCdEf
  secretPrefix: rds/prod-main
  allowedNamespaces:
    - orders
    - billing
```

Apply it:

```sh
kubectl apply -f databaseinstance.yaml
```

Important: `DatabaseInstance` is namespaced today. Create it in the same
namespace as the `DatabaseAccess` resources that reference it.

## 7. Create A DatabaseAccess

`DatabaseAccess` declares the database, schemas and application users.

```yaml
apiVersion: database.cloudvibe.dev/v1alpha1
kind: DatabaseAccess
metadata:
  name: orders-db
  namespace: orders
spec:
  instanceRef:
    name: prod-main-postgres
  database: orders
  schemas:
    - public
  users:
    - name: orders_api_rw
      permissions: readwrite
    - name: orders_api_ro
      permissions: readonly
```

Apply it:

```sh
kubectl apply -f databaseaccess.yaml
```

Check status:

```sh
kubectl -n orders get databaseaccess orders-db -o yaml
```

Expected status shape:

```yaml
status:
  phase: Ready
  database: orders
  users:
    - name: orders_api_rw
      permissions: readwrite
      secretArn: arn:aws:secretsmanager:us-east-1:123456789012:secret:rds/prod-main/orders/orders/orders_api_rw-AbCdEf
    - name: orders_api_ro
      permissions: readonly
      secretArn: arn:aws:secretsmanager:us-east-1:123456789012:secret:rds/prod-main/orders/orders/orders_api_ro-AbCdEf
```

The status never contains passwords.

## 8. Consume The Secret In Applications

The operator writes application credentials to AWS Secrets Manager. To expose
them as environment variables in Kubernetes, use External Secrets Operator.

Example `ClusterSecretStore` for IRSA:

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ClusterSecretStore
metadata:
  name: aws-secrets-manager
spec:
  provider:
    aws:
      service: SecretsManager
      region: us-east-1
      auth:
        jwt:
          serviceAccountRef:
            name: external-secrets
            namespace: external-secrets
```

Example `ExternalSecret`:

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: orders-db
  namespace: orders
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: aws-secrets-manager
    kind: ClusterSecretStore
  target:
    name: orders-db
    creationPolicy: Owner
  dataFrom:
    - extract:
        key: rds/prod-main/orders/orders/orders_api_rw
```

Example application environment:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: orders-api
  namespace: orders
spec:
  template:
    spec:
      containers:
        - name: orders-api
          image: ghcr.io/cloudvibedev/orders-api:latest
          env:
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: orders-db
                  key: uri
```

## 9. Network And Aurora Requirements

- EKS nodes or pods must be able to reach the Aurora writer endpoint on port
  `5432`.
- Aurora security groups must allow inbound PostgreSQL traffic from the EKS node
  or pod security group.
- The admin database user must be allowed to create databases, schemas, roles
  and grants.
- Use the Aurora writer endpoint for provisioning.

## 10. Troubleshooting

Check operator logs:

```sh
kubectl -n cloudvibe-system logs deploy/cloudvibe-database-operator
```

Check custom resource conditions:

```sh
kubectl -n orders describe databaseaccess orders-db
kubectl -n orders get databaseaccess orders-db -o yaml
```

Common issues:

- `InstanceNotFound`: `DatabaseInstance` is missing or is in another namespace.
- `NamespaceDenied`: the namespace is not listed in `allowedNamespaces`.
- `SecretStoreError`: the pod IAM role cannot read or write the configured
  Secrets Manager secrets.
- `ProvisioningError`: the operator cannot connect to Aurora or the admin user
  lacks database permissions.

## Production Notes

- Pin `image.tag` to a commit SHA or release tag.
- Restrict IAM resources to the exact admin secret and application prefix.
- Keep the admin secret out of Kubernetes.
- Use External Secrets Operator or another approved secret sync mechanism for
  application pods.
- Monitor OpenTelemetry traces and structured logs from the operator.

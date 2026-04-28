#!/usr/bin/env sh
set -eu

AWS_REGION="${AWS_REGION:-us-east-1}"
SECRET_ID="${ADMIN_SECRET_ID:-rds/prod-main/admin}"

docker compose -f docker-compose.e2e.yaml exec -T localstack awslocal secretsmanager create-secret \
  --region "$AWS_REGION" \
  --name "$SECRET_ID" \
  --secret-string '{"username":"postgres","password":"postgres","database":"postgres"}' \
  >/dev/null || true

docker compose -f docker-compose.e2e.yaml exec -T localstack awslocal secretsmanager put-secret-value \
  --region "$AWS_REGION" \
  --secret-id "$SECRET_ID" \
  --secret-string '{"username":"postgres","password":"postgres","database":"postgres"}' \
  >/dev/null

echo "$SECRET_ID"

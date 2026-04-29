#!/usr/bin/env sh
set -eu

COMPOSE_FILE="${COMPOSE_FILE:-docker-compose.e2e.yaml}"
KEEP_UP="${E2E_KEEP_UP:-false}"

cleanup() {
  if [ "$KEEP_UP" != "true" ]; then
    docker compose -f "$COMPOSE_FILE" down -v
  fi
}

trap cleanup EXIT

docker compose -f "$COMPOSE_FILE" up -d postgres localstack
for service in postgres localstack; do
  attempts=0
  while [ "$attempts" -lt 60 ]; do
    status="$(docker compose -f "$COMPOSE_FILE" ps --format json "$service" | grep -o '"Health":"[^"]*"' | head -1 | cut -d: -f2 | tr -d '"')"
    if [ "$status" = "healthy" ]; then
      break
    fi
    attempts=$((attempts + 1))
    sleep 2
  done
  if [ "$attempts" -eq 60 ]; then
    echo "service $service did not become healthy" >&2
    docker compose -f "$COMPOSE_FILE" logs "$service" >&2
    exit 1
  fi
done

scripts/e2e-bootstrap-localstack.sh >/dev/null

export AWS_ENDPOINT_URL="${AWS_ENDPOINT_URL:-http://localhost:4566}"
export AWS_REGION="${AWS_REGION:-us-east-1}"
export AWS_ACCESS_KEY_ID="${AWS_ACCESS_KEY_ID:-test}"
export AWS_SECRET_ACCESS_KEY="${AWS_SECRET_ACCESS_KEY:-test}"
export E2E_DATABASE_URL="${E2E_DATABASE_URL:-postgres://postgres:postgres@localhost:5432/postgres}"

cargo test --test e2e -- --ignored --nocapture

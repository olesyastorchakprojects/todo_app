#!/bin/bash

APP_CONFIG="bench"
BENCH_TEST="registration"
ITERATIONS="1000"
KDF="pbkdf2"
ENV_TOKEN_COUNT="1000"
RANDOM_IP_COUNT="0"

for ARG in "$@"; do
  case $ARG in
    --app_config=*)
      APP_CONFIG="${ARG#*=}"
      ;;
    --bench_test=*)
      BENCH_TEST="${ARG#*=}"
      ;;
    --iterations=*)
      ITERATIONS="${ARG#*=}"
      ;;
    --ips=*)
      RANDOM_IP_COUNT="${ARG#*=}"
      ;;
    --kdf=*)
      KDF="${ARG#*=}"
      ;;
    --tokens=*)
      ENV_TOKEN_COUNT="${ARG#*=}"
      ;;
    --help)
      echo "Usage: $0 [--app_config=bench] [--bench_test=registration] [--iterations=310000] [--kdf=pbkdf2] [--tokens=1000][--ips=0]"
      exit 0
      ;;
    *)
      echo "Unknown parameter: $ARG"
      exit 1
      ;;
  esac
done

case "$BENCH_TEST" in
  registration)
    SCRIPT="registration.js"
    ;;
  login)
    SCRIPT="login.js"
    ;;
  crud)
    SCRIPT="crud.js"
    ;;
  full)
    SCRIPT="full.js"
    ;;
  promote)
    SCRIPT="promote_user.js"
    ;;
  *)
    echo "Unknown BENCH_TEST: $MODE"
    echo "Acceptable values: registration, login, crud, full, promote"
    exit 1
    ;;
esac

export ENV_TOKEN_COUNT="${ENV_TOKEN_COUNT}"
export K6_SCRIPT="${SCRIPT}"
export RUN_MODE="${APP_CONFIG}"
export APP__AUTH__KDF_ALGO="${KDF}"
export APP__AUTH__PBKDF2__ITERATIONS="${ITERATIONS}"
export RANDOM_IP_COUNT="${RANDOM_IP_COUNT}"

echo "==> SCRIPT=$SCRIPT"
echo "==> APP_CONFIG=$APP_CONFIG"
echo "==> BENCH_TEST=$BENCH_TEST"
echo "==> ITERATIONS=$ITERATIONS"
echo "==> KDF=$KDF"
echo "==> ENV_TOKEN_COUNT=$ENV_TOKEN_COUNT"
echo "==> RANDOM_IP_COUNT=$RANDOM_IP_COUNT"

echo "==> Running: todo_app..."
APP__AUTH__KDF_ALGO=${KDF} APP__AUTH__PBKDF2__ITERATIONS=${ITERATIONS} RUN_MODE=${APP_CONFIG} docker compose -f docker-compose.bench.yml -p bench up app -d

if [[ "$BENCH_TEST" != "registration" ]]; then
  echo "==> Running: token_generator..."
  ENV_TOKEN_COUNT=${ENV_TOKEN_COUNT} docker compose -f docker-compose.bench.yml -p bench up token_generator
  echo "==> Generated tokens."
fi

echo "==> Running: k6..."
ENV_TOKEN_COUNT=${ENV_TOKEN_COUNT} K6_SCRIPT=${SCRIPT} RANDOM_IP_COUNT=${RANDOM_IP_COUNT} docker compose -f docker-compose.bench.yml -p bench up k6

echo "==> Stopping containers.."
docker compose -f docker-compose.bench.yml -p bench down

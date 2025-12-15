#!/bin/bash

set -e

cd "$(dirname "$0")/.."

TARGET_VOLUME="wm-test-target"
CARGO_VOLUME="wm-test-cargo"

docker volume inspect "$TARGET_VOLUME" >/dev/null 2>&1 || docker volume create "$TARGET_VOLUME"
docker volume inspect "$CARGO_VOLUME" >/dev/null 2>&1 || docker volume create "$CARGO_VOLUME"

if ! docker image inspect wm-test >/dev/null 2>&1; then
    echo "Building test container..."
    docker build -f Dockerfile.test -t wm-test . --quiet
fi

echo "Running tests..."
DOCKER_ARGS=(
    --rm --init
    -v "$(pwd):/app"
    -v "$TARGET_VOLUME:/app/target"
    -v "$CARGO_VOLUME:/root/.cargo"
)

if [ -t 1 ]; then
    docker run -t "${DOCKER_ARGS[@]}" wm-test "$@"
else
    docker run "${DOCKER_ARGS[@]}" wm-test "$@"
fi

echo "Tests completed."

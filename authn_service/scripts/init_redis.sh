# Check if a custom port has been set, otherwise default to '6379'
REDIS_PORT="${REDIS_PORT:=6378}"
# Check if a custom host has been set, otherwise default to 'localhost'
REDIS_HOST="${REDIS_HOST:=localhost}"
# Run a specific version
REDIS_VERSION="${REDIS_VERSION:="6.0.9"}"

# Allow to skip Docker if a dockerized Redis instance is running
if [[ -z "${SKIP_DOCKER}" ]]
then
  # if a redis container is running, print instructions to kill it and exit
  RUNNING_REDIS_CONTAINER=$(docker ps --filter 'name=redis_ccc_auth' --format '{{.ID}}')
  if [[ -n $RUNNING_REDIS_CONTAINER ]]; then
    echo >&2 "there is a redis container already running, kill it with"
    echo >&2 "    docker kill ${RUNNING_REDIS_CONTAINER}"
    exit 1
  fi
  # Launch postgres using Docker
  docker run \
      -p "${REDIS_PORT}":6379 \
      -d \
      --name "redis_ccc_auth_$(date '+%s')" \
      redis:${REDIS_VERSION}
      # ^ Increased maximum number of connections for testing purposes
fi
# Spawn Bitcoind, and provide execution permission.
docker compose up -d

echo "Waiting for Bitcoin node to initialize..."

# Function to check if the Bitcoin node is ready
check_bitcoin_node() {
  curl --silent --user vasu:password --data-binary '{"jsonrpc":"1.0","method":"getblockchaininfo","params":[]}' -H 'content-type:text/plain;' http://127.0.0.1:18443/ > /dev/null
  return $?
}

# Wait until the Bitcoin node is ready
MAX_RETRIES=30
RETRY_COUNT=0
while ! check_bitcoin_node && [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
  echo "Bitcoin node not ready yet, waiting..."
  sleep 2
  RETRY_COUNT=$((RETRY_COUNT+1))
done

if [ $RETRY_COUNT -eq $MAX_RETRIES ]; then
  echo "Error: Bitcoin node did not become ready in time."
  docker-compose down
  exit 1
fi

echo "Bitcoin node is ready!"

#running the cargo project (containing the rust script)
cargo run

# Stop the docker.
docker compose down -v
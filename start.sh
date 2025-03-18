
# Spawn Bitcoind, and provide execution permission.
docker compose up -d
cargo run


# Stop the docker.
docker compose down -v
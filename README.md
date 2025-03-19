# P2Pool-v2 Competency Test

This project contains a Rust script that interacts with a Bitcoin node running in regtest mode via JSON-RPC. The node is set up using Docker, and everything can be started and stopped with a single command.

## Prerequisites

- Linux OS
- Docker and Docker Compose installed
- Rust (Cargo included) installed

### Install Docker
If you don't have Docker installed, follow the instructions for your Linux distribution:
- **Ubuntu/Debian:**
  ```bash
  sudo apt-get update
  sudo apt-get install -y docker.io docker-compose
  ```
- **Fedora:**
  ```bash
  sudo dnf install -y docker docker-compose
  ```
After installation, make sure Docker is running:
```bash
sudo systemctl start docker
sudo systemctl enable docker
```

### Install Rust and Cargo
If Rust isn't installed, you can install it with Rustup:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```
For more details, visit the official Rust site: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

## Directory Structure
```
.
├── bitcoin.conf
├── Cargo.toml
├── docker-compose.yaml
├── README.md
├── rust
│   ├── Cargo.toml
│   └── src
│       └── main.rs
└── start.sh

```

## Getting Started

### 1. Clone the repository
```bash
git clone https://github.com/virtualvasu/p2pool-v2-comptests.git
cd p2pool-v2-comptests
```

### 2. Run the script
Execute everything with a single command:
```bash
./start.sh
```
This will:
- Build and run the Docker container with the Bitcoin node in regtest mode
- Compile and run the Rust script
- Stop and clean up the Docker container


## Configuration

### Bitcoin Node
The `bitcoin.conf` file is included for configuring the Bitcoin node inside the Docker container. By default, it runs in regtest mode and exposes JSON-RPC on the container.

### Docker Setup
The `docker-compose.yaml` file ensures the node runs in an isolated environment.

## Notes
- The `start.sh` script automates everything, ensuring the node starts and stops cleanly.
- Ensure Docker and Rust/Cargo are properly installed before running the script.

Happy hacking!


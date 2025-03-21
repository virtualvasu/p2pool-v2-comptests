#!/bin/bash
#
# Bitcoin Transaction Test Suite for Competency tests
# Tests transaction creation and confirmation using Rust implementation

set -e

# Bitcoin RPC config
RPC_USER="vasu"
RPC_PASS="password"
RPC_PORT="18443"
RPC_HOST="127.0.0.1"

# Test params
MAX_RETRIES=30
RETRY_WAIT=2

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
BOLD='\033[1m'
NC='\033[0m'

# Test counters
TOTAL=0
PASSED=0
FAILED=0

# Helper functions
btc_rpc() {
  curl -s --user ${RPC_USER}:${RPC_PASS} \
    --data-binary "{\"jsonrpc\":\"1.0\",\"method\":\"$1\",\"params\":[$2]}" \
    -H 'content-type:text/plain;' \
    http://${RPC_HOST}:${RPC_PORT}/
}

log_info() {
  echo -e "[$(date "+%Y-%m-%d %H:%M:%S")] ${BLUE}[INFO]${NC} $1"
}

log_success() {
  echo -e "[$(date "+%Y-%m-%d %H:%M:%S")] ${GREEN}[SUCCESS]${NC} $1"
}

log_error() {
  echo -e "[$(date "+%Y-%m-%d %H:%M:%S")] ${RED}[ERROR]${NC} $1"
}

log_header() {
  echo -e "\n${BOLD}${BLUE}=== $1 ===${NC}\n"
}

# Test assertion helper
check_test() {
  local name=$1
  local condition=$2
  local success=$3
  local failure=$4
  
  TOTAL=$((TOTAL+1))
  
  if eval "$condition"; then
    log_success "✅ $name - $success"
    PASSED=$((PASSED+1))
    return 0
  else
    log_error "❌ $name - $failure"
    FAILED=$((FAILED+1))
    return 1
  fi
}

# Check if Bitcoin node is responding
is_btc_ready() {
  btc_rpc "getblockchaininfo" "" > /dev/null
  return $?
}

# Cleanup handler
cleanup() {
  log_info "Cleaning up..."
  docker compose down -v >/dev/null 2>&1 || true
}

trap cleanup EXIT

# Print header
echo -e "\n${BOLD}${BLUE}BITCOIN TRANSACTION VALIDATION SUITE${NC}\n"
echo -e "Tests: node init, tx creation, confirmation, chain integrity, and balances\n"

# Start environment
log_header "SETUP"
log_info "Starting Bitcoin node..."
docker compose up -d

# Wait for Bitcoin node
log_info "Waiting for node to initialize..."
RETRIES=0
echo -n "  Progress: ["
while ! is_btc_ready && [ $RETRIES -lt $MAX_RETRIES ]; do
  echo -n "#"
  sleep $RETRY_WAIT
  RETRIES=$((RETRIES+1))
done
echo "] Done"

# Check node is up
check_test "Node Initialization" \
  "[ $RETRIES -lt $MAX_RETRIES ]" \
  "Bitcoin node is ready" \
  "Node failed to initialize in time"

if [ $? -ne 0 ]; then
  log_error "Critical failure: Cannot proceed without Bitcoin node"
  exit 1
fi

# Run Rust app
log_header "TRANSACTION CREATION"
log_info "Running Rust app..."

echo -e "${YELLOW}----------------------------------------${NC}"
rust_output=$(cargo run 2>&1 | tee /dev/tty)
echo -e "${YELLOW}----------------------------------------${NC}"

# Get transaction IDs
tx1=$(echo "$rust_output" | grep -oP 'First transaction ID: \K\w+')
tx2=$(echo "$rust_output" | grep -oP 'Second transaction ID: \K\w+')

# Make sure we got the txids
check_test "TX ID Extraction" \
  "[ -n \"$tx1\" ] && [ -n \"$tx2\" ]" \
  "Got transaction IDs" \
  "Failed to get transaction IDs"

if [ $? -ne 0 ]; then
  log_error "Critical failure: Can't continue without transaction IDs"
  exit 1
fi

log_info "TX IDs: ${tx1:0:8}...${tx1: -8} and ${tx2:0:8}...${tx2: -8}"

# Validate transactions
log_header "TRANSACTION VALIDATION"

# Check TX1
log_info "Checking Transaction 1..."
tx1_data=$(btc_rpc "getrawtransaction" "\"$tx1\", true")

# Confirm TX1
conf1=$(echo "$tx1_data" | jq -r '.result.confirmations // 0')
check_test "TX1 Confirmed" \
  "[ \"$conf1\" -gt 0 ]" \
  "Transaction 1 confirmed ($conf1 confirmations)" \
  "Transaction 1 not confirmed"

# Get TX1 input
input1=$(echo "$tx1_data" | jq -r '.result.vin[0].txid // empty')

# Check TX1 source
if [ -z "$input1" ] || [ "$input1" == "null" ]; then
  # Check for coinbase
  coinbase1=$(echo "$tx1_data" | jq -r '.result.vin[0].coinbase // empty')
  
  check_test "TX1 Source" \
    "[ -n \"$coinbase1\" ]" \
    "TX1 is a coinbase tx" \
    "Can't determine TX1 source"
else
  log_info "TX1 input: ${input1:0:8}...${input1: -8}"
  
  # Check if input is coinbase
  input1_data=$(btc_rpc "getrawtransaction" "\"$input1\", true")
  is_coinbase=$(echo "$input1_data" | jq -r '.result.vin[0].coinbase // empty')
  
  if [ -n "$is_coinbase" ]; then
    log_success "TX1 source is a coinbase tx"
  else
    log_info "TX1 source is not a coinbase tx"
  fi
fi

# Get TX1 details
tx1_value=$(echo "$tx1_data" | jq -r '.result.vout | map(.value) | add')
tx1_addr=$(echo "$tx1_data" | jq -r '.result.vout[0].scriptPubKey.address // "N/A"')
log_info "TX1: $tx1_value BTC to $tx1_addr"

# Check TX2
log_info "Checking Transaction 2..."
tx2_data=$(btc_rpc "getrawtransaction" "\"$tx2\", true")

# Confirm TX2
conf2=$(echo "$tx2_data" | jq -r '.result.confirmations // 0')
check_test "TX2 Confirmed" \
  "[ \"$conf2\" -gt 0 ]" \
  "Transaction 2 confirmed ($conf2 confirmations)" \
  "Transaction 2 not confirmed"

# Get TX2 input
input2=$(echo "$tx2_data" | jq -r '.result.vin[0].txid // empty')

# Check TX2 source
if [ -z "$input2" ] || [ "$input2" == "null" ]; then
  # Check for coinbase
  coinbase2=$(echo "$tx2_data" | jq -r '.result.vin[0].coinbase // empty')
  
  check_test "TX2 Source" \
    "[ -n \"$coinbase2\" ]" \
    "TX2 is a coinbase tx" \
    "Can't determine TX2 source"
else
  log_info "TX2 input: ${input2:0:8}...${input2: -8}"
  
  # Check if TX1 is input to TX2
  check_test "Transaction Chain" \
    "[ \"$input2\" == \"$tx1\" ]" \
    "Chain verified: TX1 is input to TX2" \
    "Chain broken: TX1 is not input to TX2"
  
  # Get TX2 details
  tx2_value=$(echo "$tx2_data" | jq -r '.result.vout | map(.value) | add')
  tx2_addr=$(echo "$tx2_data" | jq -r '.result.vout[0].scriptPubKey.address // "N/A"')
  log_info "TX2: $tx2_value BTC to $tx2_addr"
fi

# Check wallet balance
log_header "FINAL CHECK"
balance=$(btc_rpc "getbalance" "" | jq '.result')
log_info "Final balance: $balance BTC"

# Show summary
log_header "RESULTS"
echo -e "Tests: ${BOLD}${TOTAL}${NC}"
echo -e "Passed: ${BOLD}${GREEN}${PASSED}${NC}"
echo -e "Failed: ${BOLD}${RED}${FAILED}${NC}"

# Final result
if [ $PASSED -eq $TOTAL ]; then
  echo -e "\n${GREEN}All tests passed!${NC}\n"
  exit 0
else
  echo -e "\n${RED}Test suite failed${NC}\n"
  exit 1
fi
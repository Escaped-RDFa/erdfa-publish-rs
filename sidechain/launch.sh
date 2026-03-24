#!/usr/bin/env bash
# launch.sh — Start eRDFa Solana sidechain + stego-gossip P2P layer
set -euo pipefail

LEDGER="${LEDGER:-/var/lib/erdfa-sidechain/ledger}"
RPC_PORT="${RPC_PORT:-8899}"
GOSSIP_PORT="${GOSSIP_PORT:-7700}"
FAUCET_PORT="${FAUCET_PORT:-9900}"
WATCH_DIR="${WATCH_DIR:-/var/lib/erdfa-sidechain/inbox}"
PEERS_FILE="${PEERS_FILE:-/var/lib/erdfa-sidechain/peers.json}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

mkdir -p "$LEDGER" "$WATCH_DIR"
[ -f "$PEERS_FILE" ] || echo '[]' > "$PEERS_FILE"

echo "◎ eRDFa Solana Sidechain"
echo "========================"
echo "  Ledger:     $LEDGER"
echo "  RPC:        http://127.0.0.1:$RPC_PORT"
echo "  Gossip UDP: 0.0.0.0:$GOSSIP_PORT"
echo "  Faucet:     http://127.0.0.1:$FAUCET_PORT"
echo "  Watch:      $WATCH_DIR"
echo "  Peers:      $PEERS_FILE"
echo ""

# Configure solana CLI for local validator
solana config set --url "http://127.0.0.1:$RPC_PORT" --keypair ~/.config/solana/id.json 2>/dev/null || true

# Generate keypair if missing
if [ ! -f ~/.config/solana/id.json ]; then
    echo "Generating keypair..."
    solana-keygen new --no-bip39-passphrase -o ~/.config/solana/id.json
fi

cleanup() {
    echo ""
    echo "Shutting down..."
    kill $VALIDATOR_PID $GOSSIP_PID 2>/dev/null || true
    wait 2>/dev/null || true
    echo "Done."
}
trap cleanup EXIT

# Start Solana test-validator
echo "Starting solana-test-validator..."
solana-test-validator \
    --ledger "$LEDGER" \
    --rpc-port "$RPC_PORT" \
    --gossip-port 8001 \
    --faucet-port "$FAUCET_PORT" \
    --log - &
VALIDATOR_PID=$!

# Wait for validator to be ready
echo "Waiting for validator..."
for i in $(seq 1 30); do
    if solana --url "http://127.0.0.1:$RPC_PORT" cluster-version 2>/dev/null; then
        break
    fi
    sleep 1
done

# Airdrop some SOL for memo transactions
echo "Airdropping SOL..."
solana --url "http://127.0.0.1:$RPC_PORT" airdrop 100 2>/dev/null || true

# Start stego-gossip
echo "Starting stego-gossip on :$GOSSIP_PORT..."
"$ROOT_DIR/target/release/stego-gossip" \
    --listen "0.0.0.0:$GOSSIP_PORT" \
    --rpc "http://127.0.0.1:$RPC_PORT" \
    --peers-file "$PEERS_FILE" \
    --watch "$WATCH_DIR" &
GOSSIP_PID=$!

echo ""
echo "◎ Sidechain running!"
echo "  Drop .cbor files into $WATCH_DIR to gossip them"
echo "  Add peers: echo '[\"1.2.3.4:7700\"]' > $PEERS_FILE"
echo "  View logs: journalctl -f -u erdfa-sidechain"
echo ""

wait

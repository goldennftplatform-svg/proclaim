#!/usr/bin/env bash
# SPDX-License-Identifier: MIT AND Apache-2.0
# Automatic LP-0023 lifecycle against a fresh LEZ standalone sequencer.
set -euo pipefail

PID="${REGISTRY_PROGRAM_ID:?set REGISTRY_PROGRAM_ID}"
BASE58_PID="6FMaVQ6CmbAVNHgwsmmeBe5kiiKkQRFtqu4GUQ2vAf2T"
INDEX_PDA="9mr5hePKo24ZHNztzVpuRND7PeAsR17yQ2M5HHPKcbw3"
ENTRY_PDA="EP644ahCbBehPMV8krDKDnToxJwfaMGaKKa1NzDFcM6q"
CHANNEL_ID="${E2E_CHANNEL_ID:-0101010101010101010101010101010101010101010101010101010101010101}"
PORT="${E2E_SEQUENCER_PORT:-3040}"
BIN="methods/guest/target/riscv32im-risc0-zkvm-elf/docker/registry.bin"

# Disposable, public LEZ v0.2.4 debug identities from the LEZ Justfile.
AUTHOR="Public/CbgR6tj5kWx5oziiFptM7jMvrQeYY3Mzaao6ciuhSr2r"
AUTHOR_KEY="7f273098f25b71e6c005a9519f2678da8d1c7f01f6a27778e2d9948abdf901fb"
VERIFIER="Public/2RHZhw9h534Zr3eq2RGhQete2Hh667foECzXPmSkGni2"
VERIFIER_KEY="f434f8741720014586ae43356d2aec6257da086222f604ddb75d69733b86fc4c"

IDL_CID="${E2E_IDL_CID:?set E2E_IDL_CID}"
CODE_CID="${E2E_CODE_CID:?set E2E_CODE_CID}"
EVIDENCE_CID="${E2E_EVIDENCE_CID:?set E2E_EVIDENCE_CID}"
COMMIT="${GITHUB_SHA:-$(git rev-parse HEAD)}"
BUILD_CONFIG="guest=local,risc0=3.0.5,target=riscv32im-risc0-zkvm-elf"
API_BASE="http://127.0.0.1:${REGISTRY_API_ADDR##*:}"

mkdir -p e2e-artifacts

RPC_URL=""
NODE_REF=""
API_PID=""
cleanup() {
  if [ -n "$API_PID" ]; then
    kill "$API_PID" 2>/dev/null || true
  fi
  if [ -n "$NODE_REF" ]; then
    lgs test-node stop --node "$NODE_REF" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

step() { printf '\n== %s ==\n' "$1"; }

step "Start fresh real standalone sequencer on :$PORT"
NODE_JSON=$(lgs test-node start \
  --port "$PORT" \
  --timeout-sec 90 \
  --block-create-timeout-ms 15000 \
  --retry-pending-blocks-timeout-ms 5000 \
  --json | tee e2e-artifacts/node.json)
NODE_REF=$(printf '%s\n' "$NODE_JSON" | jq -r '.state_dir')
RPC_URL=$(printf '%s\n' "$NODE_JSON" | jq -r '.rpc_url')
test -n "$NODE_REF" && test "$NODE_REF" != null
test -n "$RPC_URL" && test "$RPC_URL" != null

lgs test-node status --node "$NODE_REF" --json | tee e2e-artifacts/node-status.json

# User-verified LEZ JSON-RPC readiness probe (not /health; that is registry-api).
curl --fail --silent --show-error --max-time 10 \
  -X POST "$RPC_URL/" \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"getChannelId","params":{}}' \
  | tee e2e-artifacts/channel-id.json \
  | jq -e --arg c "$CHANNEL_ID" '.result == $c'

step "Point wallet home at the fresh sequencer and import disposable accounts"
test -f .scaffold/wallet/wallet_config.json
jq --arg url "$RPC_URL" '.sequencers[0].sequencer_addr = $url' \
  .scaffold/wallet/wallet_config.json > e2e-artifacts/wallet_config.json
mv e2e-artifacts/wallet_config.json .scaffold/wallet/wallet_config.json

lgs wallet -- account import public --private-key "$AUTHOR_KEY"
lgs wallet -- vault claim --account-id "$AUTHOR" --amount 10000
lgs wallet -- account import public --private-key "$VERIFIER_KEY"
lgs wallet -- vault claim --account-id "$VERIFIER" --amount 20000
lgs wallet default set "$AUTHOR"
lgs wallet list --json | tee e2e-artifacts/wallet-list.json

step "Deploy and full instruction lifecycle"
lgs wallet -- deploy-program "$BIN"
lgs spel -- init-index --authority "$AUTHOR"

lgs spel -- register --key "$PID" \
  --name "LP-0023 CI Registry" \
  --version "0.3.0" \
  --description "Automatic E2E registry entry" \
  --tags "registry,spel,lez,search,verified" \
  --idl-cid "$IDL_CID" \
  --code-cid "$CODE_CID" \
  --commit "$COMMIT" \
  --build-config "$BUILD_CONFIG" \
  --timestamp "$(date +%s)" \
  --deployer-claimed true \
  --author "$AUTHOR"

lgs spel -- update --key "$PID" \
  --version "0.3.1" \
  --description "Automatic E2E registry entry after update" \
  --tags "registry,spel,lez,search,verified,updated" \
  --idl-cid "$IDL_CID" \
  --code-cid "$CODE_CID" \
  --commit "$COMMIT" \
  --build-config "$BUILD_CONFIG" \
  --author "$AUTHOR"

lgs spel -- search | tee e2e-artifacts/spel-search.txt
grep -F "LP-0023 CI Registry" e2e-artifacts/spel-search.txt

PDA_ENTRY=$(lgs spel -- pda entry --key "$PID" | tr -d '\r' | tail -n 1)
PDA_INDEX=$(lgs spel -- pda index | tr -d '\r' | tail -n 1)
test "$PDA_INDEX" = "$INDEX_PDA"
test "$PDA_ENTRY" = "$ENTRY_PDA"

lgs spel -- inspect "$PDA_INDEX" --type RegistryIndex \
  | tee e2e-artifacts/index-before-verify.txt
lgs spel -- inspect "$PDA_ENTRY" --type RegistryEntry \
  | tee e2e-artifacts/entry-before-verify.txt

lgs spel -- mark-verified --key "$PID" \
  --method "reproducible-rebuild:program-id" \
  --evidence-cid "$EVIDENCE_CID" \
  --verifier "$VERIFIER"

lgs spel -- inspect "$PDA_ENTRY" --type RegistryEntry \
  | tee e2e-artifacts/entry-after-verify.txt

step "Browser reads from the same fresh state"
cargo build -p registry-examples --bin registry_browser
./target/debug/registry_browser --program-id "$PID" --index-only --json \
  | tee e2e-artifacts/browser-index.json \
  | jq -e --arg p "$BASE58_PID" 'index($p) != null'
./target/debug/registry_browser --program-id "$PID" \
  --name "LP-0023 CI Registry" --tag registry --verified --json \
  | tee e2e-artifacts/browser-search.json \
  | jq -e '[.[] | select(.name == "LP-0023 CI Registry")] | length >= 1'

step "Read-only HTTP API backed by the same wallet/sequencer"
cargo build -p registry-api --bin registry-api
REGISTRY_API_ADDR="${REGISTRY_API_ADDR:-127.0.0.1:3030}" \
  ./target/debug/registry-api > e2e-artifacts/api.log 2>&1 &
API_PID=$!

READY=0
for _ in $(seq 1 30); do
  if curl --fail --silent --show-error --max-time 3 \
      "$API_BASE/health" > e2e-artifacts/health.json; then
    READY=1
    break
  fi
  sleep 1
done
test "$READY" = 1
jq -e '.status == "ok"' e2e-artifacts/health.json

curl --fail --silent --show-error --max-time 10 "$API_BASE/index" \
  | tee e2e-artifacts/api-index.json \
  | jq -e --arg p "$BASE58_PID" 'index($p) != null'
curl --fail --silent --show-error --max-time 10 "$API_BASE/entries" \
  | tee e2e-artifacts/api-entries.json \
  | jq -e 'length >= 1'
curl --fail --silent --show-error --max-time 10 --get "$API_BASE/search" \
  --data-urlencode "name=LP-0023 CI Registry" \
  --data-urlencode "tag=registry" \
  --data-urlencode "verified=true" \
  --data-urlencode "limit=10" \
  | tee e2e-artifacts/api-search.json \
  | jq -e 'length >= 1'
curl --fail --silent --show-error --max-time 10 \
  "$API_BASE/entry/$BASE58_PID" \
  | tee e2e-artifacts/api-entry.json \
  | jq -e --arg cid "$EVIDENCE_CID" \
      '.source_verified == true and .evidence_cid == $cid'

step "E2E complete; teardown runs via trap"

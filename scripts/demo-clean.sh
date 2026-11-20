#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
#
# Clean-environment demo for the LP-0023 LEZ Program Registry (proclaim).
#
# Offline mode (default) runs only deterministic checks that need no sequencer:
#   - toolchain presence
#   - IDL regenerates byte-for-byte from the SPEL source
#   - registry_core unit tests
#   - host crates compile
#   - JS client unit tests
#
# Online mode (--online) additionally reads a real sequencer via the configured
# wallet home and exercises the read-only API and browser. It never submits a
# transaction and never needs a private key beyond the wallet config used for
# reads.
#
# Usage:
#   scripts/demo-clean.sh
#   scripts/demo-clean.sh --online
#   scripts/demo-clean.sh --source-dir /path/to/frozen/source \
#       --program-id <64-hex> --evidence-cid <cid> --online

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

ONLINE=0
SOURCE_DIR=""
PROGRAM_ID="4df7d5396b68a16aae6d0f768ef447a0055eb43866827154f71139e486265144"
EVIDENCE_CID="zDv*8iXZ6U"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --online) ONLINE=1; shift ;;
    --source-dir) SOURCE_DIR="${2:?--source-dir needs a value}"; shift 2 ;;
    --program-id) PROGRAM_ID="${2:?--program-id needs a value}"; shift 2 ;;
    --evidence-cid) EVIDENCE_CID="${2:?--evidence-cid needs a value}"; shift 2 ;;
    -h|--help) sed -n '2,24p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

step() { printf '\n== %s ==\n' "$1"; }
ok() { printf 'OK: %s\n' "$1"; }
fail() { printf 'FAIL: %s\n' "$1" >&2; exit 1; }
skip() { printf 'SKIP: %s\n' "$1"; }

# Match the local build environment used for the deployed artifact.
if [ -f "$HOME/.cargo/env" ]; then
  # shellcheck disable=SC1091
  source "$HOME/.cargo/env"
fi
export RUSTFLAGS="${RUSTFLAGS:---cfg getrandom_backend=\"custom\"}"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"

step "1/6 toolchain"
command -v cargo >/dev/null 2>&1 || fail "cargo not found (install Rust and source ~/.cargo/env)"
rustc --version
cargo --version
# On WSL the Windows Node install is often on the PATH as node.exe only.
NODE_BIN="$(command -v node 2>/dev/null || command -v node.exe 2>/dev/null || true)"
if [ -n "$NODE_BIN" ]; then
  NODE_BIN="$(command -v node 2>/dev/null || true)"
  [ -n "$NODE_BIN" ] || NODE_BIN="$(command -v node.exe)"
  "$NODE_BIN" --version
else
  skip "node not found; JS client tests cannot run"
fi
ok "toolchain"

step "2/6 IDL regenerates from SPEL source"
IDL_TMP="$(mktemp)"
trap 'rm -f "$IDL_TMP"' EXIT
cargo run --quiet --bin generate_idl > "$IDL_TMP"
diff -u registry-idl.json "$IDL_TMP" || fail "registry-idl.json is stale; run 'make idl' and commit"
ok "IDL byte-for-byte reproducible"

step "3/6 registry_core unit tests"
cargo test -p registry_core
ok "SDK tests passed"

step "4/6 host crates compile"
cargo check -p registry-api -p registry-examples --all-targets
ok "host crates compile"

step "5/6 JS client unit tests"
if [ -n "${NODE_BIN:-}" ]; then
  "$NODE_BIN" --test clients/js/test/registry.test.js
  ok "JS client tests passed"
else
  skip "JS client tests (node unavailable)"
fi

step "6/6 source verification"
if [ -n "$SOURCE_DIR" ] && [ -d "$SOURCE_DIR" ]; then
  scripts/verify-source.sh "$PROGRAM_ID" "$SOURCE_DIR" "$EVIDENCE_CID"
  ok "clean-room rebuild reproduced the program ID"
else
  skip "source verification (pass --source-dir <frozen source tree> to enable)"
fi

if [ "$ONLINE" -eq 1 ]; then
  step "ONLINE: read the real sequencer"
  : "${LEE_WALLET_HOME_DIR:?set LEE_WALLET_HOME_DIR before --online}"
  cargo build -p registry-examples --bin registry_browser
  ./target/debug/registry_browser --index-only
  ./target/debug/registry_browser --verified --json

  step "ONLINE: read-only HTTP API smoke test"
  REGISTRY_API_ADDR=127.0.0.1:3030 cargo run -p registry-api --bin registry-api >/tmp/registry-api-demo.log 2>&1 &
  API_PID=$!
  trap 'rm -f "$IDL_TMP"; kill "$API_PID" 2>/dev/null || true' EXIT
  for _ in 1 2 3 4 5 6 7 8 9 10; do
    if curl -fsS http://127.0.0.1:3030/health >/dev/null 2>&1; then break; fi
    sleep 1
  done
  curl -fsS http://127.0.0.1:3030/health
  printf '\n'
  curl -fsS 'http://127.0.0.1:3030/search?name=registry&verified=true' | head -c 400
  printf '\n'
  kill "$API_PID" 2>/dev/null || true
  ok "online read-only checks passed"
else
  step "ONLINE checks"
  skip "sequencer/API checks (pass --online with LEE_WALLET_HOME_DIR set)"
fi

printf '\nDemo complete. Offline checks are deterministic; online checks are read-only.\n'

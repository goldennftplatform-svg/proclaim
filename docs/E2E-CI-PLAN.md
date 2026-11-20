# E2E-CI-PLAN.md — automatic green real-sequencer E2E for LP-0023

Scope: **research/plan only**. This document does not edit the `proclaim` repository, commit, push, use tokens, or open PRs. It is the single planned output for closing the pending LP-0023 criterion “E2E integration tests against a real sequencer in CI, green on the default branch”. Date: 2026-11-19.

## Executive summary (8 lines)

1. Replace the manual `workflow_dispatch`-only `e2e` job with an always-on `e2e` job on every `push` and `pull_request`, built from pinned LEZ v0.2.4/SPEL v0.7.0 source.
2. The job uses `lgs setup` + `lgs test-node prepare/doctor/start --port 3040` to run a fresh **real standalone LEZ sequencer**; no public endpoint, Bedrock/Indexer stack, or private wallet secret is needed.
3. Verified CLI facts: sequencer config path is positional with `-p/--port`, `--listen-address`, `--home`, `--metrics-address`; `--features standalone` swaps in mock clients; wallet uses `account import public --private-key`, `vault claim --account-id/--amount`, `deploy-program <bin>`.
4. Verified CLI facts: SPEL uses `program-id <bin> --format hex`, `pda entry --key`, `pda index`, `inspect <account> --type`, and the IDL-driven flags `init-index --authority`, `register/update --key/--author`, `mark-verified --key/--verifier`; browser/API flags and routes are source-confirmed.
5. The script builds the guest locally with `cargo +risc0`, wraps the ELF in R0BF with a pinned `make-bin` helper, and asserts program ID `4df7d5396b68a16aae6d0f768ef447a0055eb43866827154f71139e486265144`.
6. It then runs deploy → init-index → register → update → search → mark-verified, and asserts browser `--json`, API `/health`, `/index`, `/entries`, `/search`, and `/entry/:program_id` read the freshly written state.
7. Caching is split between `Swatinem/rust-cache` for the registry workspace and `actions/cache` for `LOGOS_SCAFFOLD_CACHE_ROOT`; job timeout is 60 min, node startup 90 s, API readiness 30 s.
8. Unverified until a real GHA proof run: cold-run build times, exact `lgs spel --`/`lgs wallet --` passthrough edge cases, fresh-run guest-ID reproducibility, `make-bin` public provenance, and whether `RISC0_DEV_MODE=1` is acceptable proof-wise.

---

## 1. Authoritative local sources used

All CLI facts below were read from the local pinned checkouts, not inferred from memory or public guessing.

| Topic | Local authoritative source |
| --- | --- |
| Registry pins | `\\wsl$\Ubuntu\home\xps\logos-work\registry\scaffold.toml` |
| Registry Cargo.lock pin | `\\wsl$\Ubuntu\home\xps\logos-work\registry\Cargo.lock` |
| IDL-derived lifecycle flags | `\\wsl$\Ubuntu\home\xps\logos-work\registry\registry-idl.json` |
| Registry Makefile commands | `\\wsl$\Ubuntu\home\xps\logos-work\registry\Makefile` |
| Browser flags/output | `C:\Users\XPS\Downloads\GFD\staged\registry_browser.rs` and repo `examples/src/bin/registry_browser.rs` |
| API routes/env/shape | `C:\Users\XPS\Downloads\GFD\staged\registry_api_main.rs` and repo `registry-api/src/main.rs` |
| Current CI status | `C:\Users\XPS\Downloads\GFD\staged\ci.yml`, `CI.md` |
| Proven local lifecycle | `C:\Users\XPS\Downloads\GFD\LP0023.md` |
| LEZ v0.2.4 source root | `\\wsl$\Ubuntu\home\xps\.cargo\git\checkouts\logos-execution-zone-6bae42d7c9cadfe7\47eba25\` |
| LEZ standalone feature | LEZ `Cargo.toml`: `standalone = ["sequencer_core/mock"]`; `lez/sequencer/service/src/lib.rs` selects `SequencerCoreWithMockClients`; `Justfile` uses `cargo run --features standalone --release -p sequencer_service -- configs/debug/sequencer_config.json`. |
| Sequencer CLI | LEZ `lez/sequencer/service/src/main.rs`: positional `config`, `-p/--port` default `3040`, `--listen-address` default `0.0.0.0`, `--home`, `--metrics-address`. |
| Sequencer RPC readiness | LEZ `lez/sequencer/service/rpc/src/lib.rs`: JSON-RPC method `getChannelId`. |
| Debug sequencer/wallet config | LEZ `configs/debug/sequencer_config.json` and `configs/debug/wallet_config.json`; wallet points at `http://127.0.0.1:3040`. |
| Debug funded identities | LEZ `Justfile` `wallet-import-test-accounts` recipe. |
| LEZ CI native deps/toolchain | LEZ `.github/docker/ci.Dockerfile`, `rust-toolchain.toml` (`1.94.0`). |
| `RISC0_DEV_MODE=1` meaning | LEZ `Justfile`, `README.md`, `docs/benchmarks/integration_bench.md`: skips proof generation / disables proof verification. |
| Scaffold 0.5.0 source | `\\wsl$\Ubuntu\home\xps\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\logos-scaffold-0.5.0\` |
| Scaffold test-node behavior | `src/testnode/mod.rs`, `src/testnode/pins.rs`, `src/commands/testnode.rs`, `src/sequencer_config.rs`, `src/commands/setup.rs`. |
| SPEL v0.7.0 CLI source | `\\wsl$\Ubuntu\home\xps\.cargo\git\checkouts\spel-411f019390120158\4107dcb\spel-cli\src\` (`lib.rs`, `inspect.rs`, `tx.rs`). |

### 1.1 Verified facts that drive the plan

- Pins: LEZ `47eba256479f6f785acbd138834340703cd03401` (tag `v0.2.4`), SPEL `4107dcb14ec0eb64e9e80a9691a3df0b9acd3c2b` (tag `v0.7.0`), circuits `0.4.2`, `logos-scaffold` `0.5.0`.
- `cargo build --release -p sequencer_service --features standalone` is the correct source build. The config file is a **runtime** argument; it is not a cargo argument.
- `lgs test-node start --port 3040` patches the debug config with `port: 3040` and `max_block_size: 8 MiB`, and passes `--listen-address 127.0.0.1` because v0.2.4 advertises that flag.
- `lgs test-node start --json` prints `TestNodeInfo`: `rpc_url`, `pid`, `state_dir`, `config_path`, `log_path`, `genesis_block_id`, `block_height`. There is **no** `id` field; use `basename(.state_dir)` or the `state_dir` path itself as the `--node` selector.
- `lgs test-node status --node <selector> --json` fails the process if not healthy.
- `lgs setup` calls `ensure_circuits_for_project`, builds the standalone sequencer, wallet, `spel`, `spel-client-gen`, copies the LEZ debug wallet config into `.scaffold/wallet/wallet_config.json`, and seeds a default account. It does **not** require Docker.
- `lgs wallet --` and `lgs spel --` set both `LEE_WALLET_HOME_DIR` and `NSSA_WALLET_HOME_DIR` for the child process.
- Wallet CLI from source: `account import public --private-key <hex>`, `vault claim --account-id <Public/...> --amount <u128>`, `deploy-program <bin>` (positional), `account list`.
- IDL seeds: `index` PDA uses const `registry_index`; `entry` PDA uses arg `key`. Therefore `lgs spel -- pda index` and `lgs spel -- pda entry --key <hex>` are deterministic.
- Browser flags confirmed: `--program-id`, `--name`, `--tag` (repeatable), `--author`, `--verified`, `--json`, `--index-only`, `--limit`. `--index-only --json` emits a JSON array of base58 program IDs.
- API env/routes confirmed: `REGISTRY_API_ADDR` default `127.0.0.1:3030`, `REGISTRY_PROGRAM_ID`, `LEE_WALLET_HOME_DIR`; `/health` returns `{"status":"ok"}`; `/index`, `/entries`, `/search` (`name`, `tag`, `author`, `verified`, `limit`), `/entry/:program_id`.
- Base58 of `4df7...65144` is `6FMaVQ6CmbAVNHgwsmmeBe5kiiKkQRFtqu4GUQ2vAf2T` (recomputed locally from source `to_base58`).
- Port decision: use **3040**. LEZ debug wallet, debug sequencer default, `lgs setup` wallet config, and the user-verified running sequencer all use 3040. The `scaffold.toml` `[localnet] port = 3041` is a discrepancy that this E2E path bypasses; do not mix 3041 into the wallet/API/browser assertions.

---

## 2. Replacement `.github/workflows/ci.yml`

Replace the existing manual `e2e` job with the always-on job below. The `rust` and `js` jobs are kept unchanged for continuity.

```yaml
name: CI

on:
  push:
  pull_request:
  workflow_dispatch:

permissions:
  contents: read

env:
  CARGO_TERM_COLOR: always
  CARGO_BUILD_JOBS: "2"
  # Matches the local host build; required by the LEZ custom getrandom backend.
  RUSTFLAGS: '--cfg getrandom_backend="custom"'

jobs:
  rust:
    name: Rust — fmt, clippy, unit tests
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Cache Cargo artifacts
        uses: Swatinem/rust-cache@v2

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Clippy (host crates)
        run: cargo clippy -p registry_core -p registry-api -p registry-examples --all-targets -- -D warnings

      - name: Unit tests (registry_core)
        run: cargo test -p registry_core

      - name: Compile check (API + host tools)
        run: cargo check -p registry-api -p registry-examples --all-targets

  js:
    name: JS client tests
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v4

      - name: Install Node
        uses: actions/setup-node@v4
        with:
          node-version: '22'

      - name: Run JS client tests
        run: node --test clients/js/test/registry.test.js

  e2e:
    name: E2E — real standalone sequencer
    runs-on: ubuntu-24.04
    timeout-minutes: 60
    env:
      LEE_WALLET_HOME_DIR: ${{ github.workspace }}/.scaffold/wallet
      NSSA_WALLET_HOME_DIR: ${{ github.workspace }}/.scaffold/wallet
      LOGOS_SCAFFOLD_CACHE_ROOT: ${{ github.workspace }}/.cache/logos-scaffold
      LOGOS_SCAFFOLD_QUIET: "1"
      RUST_LOG: info
      # LEZ documents this as skipping proof generation / disabling proof
      # verification. This job tests lifecycle + reads, NOT proof verification.
      RISC0_DEV_MODE: "1"
      REGISTRY_API_ADDR: 127.0.0.1:3030
      REGISTRY_PROGRAM_ID: 4df7d5396b68a16aae6d0f768ef447a0055eb43866827154f71139e486265144
      E2E_SEQUENCER_PORT: "3040"
      E2E_CHANNEL_ID: 0101010101010101010101010101010101010101010101010101010101010101
      # Metadata strings only. Logos Storage CIDs are not fetched by this job.
      E2E_IDL_CID: ${{ vars.E2E_IDL_CID || 'zDv*8iXZ6U' }}
      E2E_CODE_CID: ${{ vars.E2E_CODE_CID || 'zDv*yXuozL' }}
      E2E_EVIDENCE_CID: ${{ vars.E2E_EVIDENCE_CID || 'zDv*8iXZ6U' }}
    steps:
      - uses: actions/checkout@v4

      - name: Prepare artifacts directory
        run: mkdir -p e2e-artifacts

      - name: Install native dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y --no-install-recommends \
            build-essential clang libclang-dev libssl-dev pkg-config \
            libpcsclite-dev curl git jq python3-dev

      - name: Install Rust 1.94.0
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: 1.94.0

      - name: Cache registry workspace build
        uses: Swatinem/rust-cache@v2
        with:
          shared-key: e2e-registry

      - name: Cache pinned LEZ/SPEL checkouts and circuits
        uses: actions/cache@v4
        with:
          path: ${{ env.LOGOS_SCAFFOLD_CACHE_ROOT }}
          key: ${{ runner.os }}-logos-scaffold-${{ hashFiles('scaffold.toml') }}
          restore-keys: |
            ${{ runner.os }}-logos-scaffold-

      - name: Install RISC Zero local toolchain
        run: |
          curl -L https://risczero.com/install | bash
          echo "$HOME/.risc0/bin" >> "$GITHUB_PATH"
          export PATH="$HOME/.risc0/bin:$PATH"
          rzup install rust 1.94.1
          rzup install r0vm 3.0.5
          rzup install cargo-risczero 3.0.5
          rustc --version
          cargo --version
          cargo +risc0 --version

      - name: Install logos-scaffold 0.5.0
        run: cargo install logos-scaffold --version 0.5.0 --locked

      - name: Verify committed IDL is reproducible
        run: |
          cargo run --quiet --bin generate_idl > /tmp/e2e-registry-idl.json
          diff -u registry-idl.json /tmp/e2e-registry-idl.json

      - name: Build pinned sequencer, wallet and spel binaries
        run: |
          lgs test-node pins --json | tee e2e-artifacts/test-node-pins.json
          jq -e '.lez_ref == "47eba256479f6f785acbd138834340703cd03401"' \
            e2e-artifacts/test-node-pins.json
          lgs setup
          lgs test-node prepare --json | tee e2e-artifacts/test-node-prepare.json
          lgs test-node doctor --json | tee e2e-artifacts/test-node-doctor.json

      - name: Build guest locally and assert deployed program ID
        run: |
          cargo +risc0 build --release \
            --manifest-path methods/guest/Cargo.toml \
            --target riscv32im-risc0-zkvm-elf
          cargo build --release --manifest-path tools/make-bin/Cargo.toml
          tools/make-bin/target/release/make-bin \
            methods/guest/target/riscv32im-risc0-zkvm-elf/release/registry \
            methods/guest/target/riscv32im-risc0-zkvm-elf/docker/registry.bin
          lgs spel -- program-id \
            methods/guest/target/riscv32im-risc0-zkvm-elf/docker/registry.bin \
            --format hex | tee e2e-artifacts/program-id.txt
          test "$(tr -d '\r' < e2e-artifacts/program-id.txt | tail -n 1)" = "$REGISTRY_PROGRAM_ID"

      - name: Run full lifecycle against fresh sequencer
        run: bash scripts/e2e-real-sequencer.sh

      - name: Upload E2E diagnostics
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: e2e-diagnostics
          path: |
            e2e-artifacts/
            .scaffold/test-nodes/
            .scaffold/wallet/
          if-no-files-found: ignore
```

### Why not `lgs build` for the guest

`logos-scaffold` 0.5.0 source states that SPEL projects always build guests through `make build` in Docker, regardless of `[build].guest`. The deployed program ID in `LP0023.md` came from a **local** `cargo +risc0 build` plus R0BF wrapping. A Docker guest build is likely to produce a different image ID and would break the required program-ID assertion. Therefore this plan deliberately uses the local guest build path recorded in `LP0023.md` and does not call `lgs build` for the guest.

---

## 3. New file: `scripts/e2e-real-sequencer.sh`

This is the single lifecycle driver. It assumes the workflow build steps have already produced the guest `.bin`, the LEZ binaries, and a seeded wallet home.

```bash
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
```

Notes on the script:

- `lgs wallet --` / `lgs spel --` set the wallet-home env vars for their children; the explicit job-level `LEE_WALLET_HOME_DIR`/`NSSA_WALLET_HOME_DIR` are still needed for `registry_browser` and `registry-api`, which are started directly.
- `lgs spel -- ...` is the documented passthrough form. It may print a deprecation warning about mixing CLI/instruction args without a literal `--`; that is cosmetic and tracked as TODO below.
- `spel inspect` output is captured for diagnostics but is **not** grepped. The verification assertion is made through browser/API JSON, whose serialization is source-confirmed.
- `scripts/demo-clean.sh --online` remains useful as a *read-only* reuse path, but it is not sufficient for the criterion because it does not start a sequencer or submit lifecycle transactions. Keep it as a local demo, not the CI gate.

---

## 4. New file: `tools/make-bin/Cargo.toml`

Pinned from the local helper used to reproduce the deployed program ID.

```toml
[package]
name = "make-bin"
version = "0.1.0"
edition = "2021"

[dependencies]
risc0-binfmt = "=3.0.5"
risc0-zkos-v1compat = "2.2.3"
```

## 5. New file: `tools/make-bin/src/main.rs`

```rust
use std::{env, fs};

use risc0_binfmt::ProgramBinary;
use risc0_zkos_v1compat::V1COMPAT_ELF;

fn main() {
    let args: Vec<String> = env::args().collect();
    let input = &args[1];
    let output = &args[2];
    let user_elf = fs::read(input).expect("read user ELF");
    let binary = ProgramBinary::new(&user_elf, V1COMPAT_ELF);
    let blob = binary.encode();
    fs::write(output, &blob).expect("write .bin");
    println!("wrote {} bytes to {}", blob.len(), output);
}
```

This helper is the one piece of local tooling not yet published in the repo. It is deliberately included here so the plan is executable; see the TODO section for provenance verification.

---

## 6. Exact env vars, caching, and timeouts

| Category | Key | Value / purpose |
| --- | --- | --- |
| Global env | `CARGO_TERM_COLOR` | `always` |
| Global env | `CARGO_BUILD_JOBS` | `2` (matches local build; avoids runner OOM) |
| Global env | `RUSTFLAGS` | `--cfg getrandom_backend="custom"` (LEZ custom getrandom backend) |
| E2E env | `LEE_WALLET_HOME_DIR` | `${{ github.workspace }}/.scaffold/wallet` |
| E2E env | `NSSA_WALLET_HOME_DIR` | same path; some SPEL errors mention this older name |
| E2E env | `LOGOS_SCAFFOLD_CACHE_ROOT` | `${{ github.workspace }}/.cache/logos-scaffold` |
| E2E env | `LOGOS_SCAFFOLD_QUIET` | `1` to keep `--json` stdout parseable |
| E2E env | `RUST_LOG` | `info` |
| E2E env | `RISC0_DEV_MODE` | `1` (fast stub receipts; **not** a proof-verification test) |
| E2E env | `REGISTRY_API_ADDR` | `127.0.0.1:3030` |
| E2E env | `REGISTRY_PROGRAM_ID` | `4df7d5396b68a16aae6d0f768ef447a0055eb43866827154f71139e486265144` |
| E2E env | `E2E_SEQUENCER_PORT` | `3040` |
| E2E env | `E2E_CHANNEL_ID` | `0101...0101` (64 hex chars) |
| E2E env | `E2E_IDL_CID` / `E2E_CODE_CID` / `E2E_EVIDENCE_CID` | repo vars with LP0023 string fallbacks; metadata only |
| Cache | `Swatinem/rust-cache@v2` | `shared-key: e2e-registry` for the root workspace |
| Cache | `actions/cache@v4` | path `LOGOS_SCAFFOLD_CACHE_ROOT`, key `${{ runner.os }}-logos-scaffold-${{ hashFiles('scaffold.toml') }}`, restore prefix by OS |
| Job timeout | `timeout-minutes` | `60` |
| Node startup | `lgs test-node start --timeout-sec` | `90` |
| Block timing | `--block-create-timeout-ms` / `--retry-pending-blocks-timeout-ms` | `15000` / `5000` (matches debug config `"15s"` / `"5s"`) |
| API readiness | script loop | 30 × 1 s, each curl `--max-time 3` |
| RPC readiness | script curl | `--max-time 10` |
| Toolchain | Rust | `1.94.0` host; `rzup install rust 1.94.1`, `r0vm 3.0.5`, `cargo-risczero 3.0.5` |
| Scaffold | `logos-scaffold` | pinned `0.5.0`, `--locked` |

---

## 7. Risks and open TODOs

1. **No GHA proof run exists.** Everything here is source-level feasible, but cold-run build times, `apt` dependency completeness, and LEZ/SPEL builds on `ubuntu-24.04` are expectations until the first green PR run.
2. **`RISC0_DEV_MODE=1` weakens proof claims.** LEZ documents it as skipping proof generation / disabling proof verification. Do not claim proof verification. TODO: decide whether a second proving-enabled job is required by the LP-0023 spec.
3. **Guest ID reproducibility in a clean runner is unverified.** The local build produced `4df7...65144`; the CI job asserts it. TODO: run once on a clean GHA runner; if the ID differs, commit the exact pinned toolchain/artifact or reconsider the assertion.
4. **`make-bin` provenance.** The helper source is local (`~/logos-work/make-bin`) and was verified from its `Cargo.toml`/`main.rs`, but no public install path was confirmed. TODO: publish/pin it in the repo (as proposed above) or replace with a documented public source.
5. **`lgs spel --` passthrough edge cases.** The `--` separator consumed by `lgs` means SPEL may emit a cosmetic deprecation warning. TODO: confirm whether `lgs spel -- -- <cmd>` cleanly forwards a literal separator; adjust if desired.
6. **`lgs wallet --` command paths** were verified from source (`account import public`, `vault claim`, `deploy-program`), but the wrapper's exact argv rendering should be checked on the first run.
7. **`spel search` output formatting** is not machine-contract-verified. The script uses `grep -F`; TODO: replace with a JSON assertion if the pinned CLI gains one.
8. **PDAs `9mr5...` / `EP644...` are asserted from LP0023 ground truth.** They are derived from const `registry_index` and arg `key`, so they should hold, but TODO: confirm on the fresh chain during the proof run.
9. **Port discrepancy (3040 vs 3041).** This plan standardizes on 3040. TODO: reconcile `scaffold.toml [localnet] port = 3041` in a separate cleanup so future `lgs localnet` users are not confused.
10. **Disposable identities.** The two author/verifier keys are public LEZ debug values, not real funds. TODO: confirm the prize/repo policy accepts them; otherwise supply an equivalent no-value wallet home without adding a secret.
11. **Local LP0023 identities differ** (`Public/HQk3...` author, `Public/CpSH...` verifier). That is fine because the CI chain is fresh; do not mix those IDs with the CI debug identities.
12. **CIDs are metadata strings.** Logos Storage CIDs in `LP0023.md` are not HTTP-fetchable from this environment; the job must not claim content availability.
13. **Fixed port 3040 fails fast if occupied.** On a dedicated GHA runner this is acceptable; TODO: switch to `lgs test-node start` auto port and rewrite the wallet config from `.rpc_url` if shared runners ever cause collisions.
14. **Docs must change only after a green default-branch run.** Update `docs/CI.md` and `docs/SOLUTION-LP-0023.md` with a linked successful run; do not check the criterion beforehand.

---

## 8. Acceptance criteria for implementation

- `e2e` runs on `push` with no branch filter and on `pull_request`; no `workflow_dispatch` gate and no `E2E_WALLET_HOME_B64` secret.
- The job starts a real LEZ v0.2.4 standalone sequencer built from the pinned commit and proves readiness with JSON-RPC `getChannelId`.
- The guest `.bin` program ID equals `4df7d5396b68a16aae6d0f768ef447a0055eb43866827154f71139e486265144`.
- Deploy → init-index → register → update → search → mark-verified all succeed, with PDA and inspect diagnostics captured.
- `registry_browser --json` and all five API routes read the freshly written state, including the verified-only filter and `evidence_cid`.
- Diagnostics upload on failure.
- At least one successful default-branch GHA run is linked before the LP-0023 criterion is marked complete.

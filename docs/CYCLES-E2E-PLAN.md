# LP-0023 closeout plan: measured cycles + automatic real-sequencer E2E CI

Scope: research/plan only. This document does not edit the `proclaim` repository,
commit, push, or use tokens. It is written for program ID:

`4df7d5396b68a16aae6d0f768ef447a0055eb43866827154f71139e486265144`

Two LP-0023 criteria are still open:

1. Documented on-chain/guest cycle counts for `register` and `update` (and the
   rest of the registry instructions).
2. Automatic green E2E integration tests against a real sequencer in CI, on
   every push, including the full lifecycle and browser/API reads.

The honest starting point is: **neither criterion can be closed with the tools
currently documented in `proclaim`.** The cycle command currently proposed in
`staged/PERFORMANCE.md` is unsupported by the authoritative LEZ source, and the
current `.github/workflows/ci.yml` E2E job is manual and read-only. The plan
below closes both gaps without inventing results or flags.

---

## 1. Authoritative sources used

| Topic | Authoritative source |
| --- | --- |
| LEZ repository / default development branch | https://github.com/logos-blockchain/logos-execution-zone (`dev`) |
| LEZ release pinned by `proclaim` | https://github.com/logos-blockchain/logos-execution-zone/tree/v0.2.4 |
| LEZ `cycle_bench` docs | https://github.com/logos-blockchain/logos-execution-zone/blob/dev/docs/benchmarks/cycle_bench.md |
| LEZ `cycle_bench` CLI source | https://github.com/logos-blockchain/logos-execution-zone/blob/dev/tools/cycle_bench/src/main.rs |
| LEZ `cycle_bench` Cargo features | https://github.com/logos-blockchain/logos-execution-zone/blob/dev/tools/cycle_bench/Cargo.toml |
| LEZ standalone sequencer command | https://github.com/logos-blockchain/logos-execution-zone/blob/v0.2.4/Justfile |
| Sequencer CLI/source | https://github.com/logos-blockchain/logos-execution-zone/blob/v0.2.4/lez/sequencer/service/src/main.rs |
| Sequencer debug config | https://github.com/logos-blockchain/logos-execution-zone/blob/v0.2.4/lez/sequencer/service/configs/debug/sequencer_config.json |
| LEZ wallet debug config | https://github.com/logos-blockchain/logos-execution-zone/blob/v0.2.4/lez/wallet/configs/debug/wallet_config.json |
| LEZ CI examples | https://github.com/logos-blockchain/logos-execution-zone/blob/dev/.github/workflows/ci.yml and https://github.com/logos-blockchain/logos-execution-zone/blob/dev/.github/actions/run-in-ci-image/action.yml |
| `logos-scaffold` install/commands | https://github.com/logos-co/scaffold/blob/master/README.md and https://github.com/logos-co/scaffold/blob/master/docs/commands.md |
| `logos-scaffold` config semantics | https://github.com/logos-co/scaffold/blob/master/docs/configuration.md |
| `proclaim` local lifecycle | https://github.com/goldennftplatform-svg/proclaim/blob/master/LP0023.md |
| `proclaim` integration tutorial | https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/INTEGRATION.md |
| Current `proclaim` CI status | https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/CI.md |
| Current `proclaim` workflow | https://github.com/goldennftplatform-svg/proclaim/blob/master/.github/workflows/ci.yml |
| `proclaim` scaffold pins | https://github.com/goldennftplatform-svg/proclaim/blob/master/scaffold.toml |

The `proclaim` checkout inspected for this plan is `master` commit
`a8878b1bfd55b55fa1a0d7936293525057fde3cd`. Re-check the spec and workflows
immediately before submission because new PRs can change the target.

---

## 2. Cycle-count work

### 2.1 Findings

The current `staged/PERFORMANCE.md` proposes:

```bash
cycle_bench --program-id 4df7d5396b68a16aae6d0f768ef447a0055eb43866827154f71139e486265144 \
  --instruction register --iterations 100
```

**This command is not supported and must be removed from the documentation.**
Evidence from `tools/cycle_bench/src/main.rs` on LEZ `dev`:

- The only CLI flags are `--prove`, `--ppe`, and `--exec-iters`.
- There is no `--program-id`, `--instruction`, or `--iterations` flag.
- The tool benchmarks a hard-coded list of built-in LEZ guests: `clock`,
  `fee`, `scripted`, `system_upgrader`, and `sequencer_stake`.
- An unknown program account fails with
  `no guest is registered at {program_account_id}`.
- It is a Cargo target (`cycle_bench`), not a documented standalone binary that
  accepts arbitrary deployed program IDs.

The official build/run commands from LEZ `dev` docs are:

```bash
git clone https://github.com/logos-blockchain/logos-execution-zone.git
cd logos-execution-zone

cargo run --release -p cycle_bench -- --exec-iters 50
cargo run --release -p cycle_bench --features prove -- --prove
cargo run --release -p cycle_bench --features ppe -- --prove --ppe
cargo bench -p cycle_bench --features ppe --bench verify
```

For the v0.2.4 tag pinned by `proclaim`, the equivalent documented commands are:

```bash
git clone --branch v0.2.4 --depth 1 \
  https://github.com/logos-blockchain/logos-execution-zone.git
cd logos-execution-zone

cargo run --release -p cycle_bench -- --exec-iters 50
cargo run --release -p cycle_bench --features prove -- --prove
cargo run --release -p cycle_bench --features ppe -- --prove --ppe
cargo bench -p cycle_bench --features ppe --bench verify
```

Outputs:

- `target/cycle_bench.json` from the binary.
- Version-dependent criterion output under `target/criterion/`:
  - v0.2.4: `target/criterion/ppe/verify_auth_transfer/`
  - `dev`: `target/criterion/ppe/verify_native_transfer/`

The LEZ docs report `user_cycles`, `segments`, `exec_ms`
(`best / mean ± stdev`), `calib_ms`, `net_ms`, and, with proving,
`total_cycles`, `prove_ms`, `prove_s`, `proof_bytes`, plus criterion low/point/
high estimates with 95% CI. The documented machine is Apple M2 Pro, 16 GB,
Rust 1.94.0, Risc0 zkVM 3.0.5, release, no GPU (macOS 15.5 for v0.2.4 docs;
macOS 26.5.1 on `dev`). Those numbers are for LEZ built-ins only. **None of them
are proclaim `register` or `update` cycles.**

### 2.2 Action plan for cycles

1. **Correct `docs/PERFORMANCE.md` first.** Remove the unsupported
   `cycle_bench --program-id ... --instruction ... --iterations ...` procedure.
   Replace it with the official LEZ commands above, clearly labelled as
   "built-in LEZ baseline only, not proclaim cycles".

2. **Keep `TBD` until measured.** Do not infer cycles from account reads/writes.
   The current table is honest; only replace a cell after the harness below
   produces an artifact-backed number.

3. **Add a proclaim-specific guest-cycle harness.** Because LEZ `cycle_bench`
   cannot load an external program ID, maintainers must add a small binary to
   `proclaim` (or an equivalent fork of the LEZ harness) that:
   - loads the exact R0BF artifact
     `methods/guest/target/riscv32im-risc0-zkvm-elf/docker/registry.bin`;
   - uses the deployed program ID
     `4df7...65144` as the expected image ID;
   - constructs the same root `ReceiveInput`/message and pre-state fixtures used
     by the on-chain `register` and `update` transactions (index length `N`,
     entry state, signer authorization, string/CID lengths, timestamp);
   - executes with `risc0_zkvm::default_executor().execute(env, elf)` in the same
     shape as LEZ `tools/cycle_bench/src/main.rs::sample()`;
   - records every guest transition triggered by the transaction, not only the
     root call;
   - discards one warm-up iteration and reports `cycles()`, `segments.len()`,
     best/mean/stdev, median and p95 over the remaining iterations;
   - optionally records `ProveInfo.stats.total_cycles`, `user_cycles`,
     `paging_cycles`, `segments`, and wall time in a separate proving run;
   - writes a machine-readable JSON artifact and prints a Markdown table row.

   Do **not** invent a CLI contract in the docs before this binary exists. When
   the harness is implemented, the actual command used by the harness must be
   copied verbatim into `docs/PERFORMANCE.md`.

4. **Build the measured ELF from published source.** Use the documented local
   pathway until a reproducible container is published:

   ```bash
   . ~/.cargo/env
   export PATH="$HOME/.risc0/bin:$PATH"
   export CARGO_BUILD_JOBS=2
   export RUSTFLAGS='--cfg getrandom_backend="custom"'

   cargo +risc0 build --release \
     --manifest-path methods/guest/Cargo.toml \
     --target riscv32im-risc0-zkvm-elf

   "$MAKE_BIN" \
     methods/guest/target/riscv32im-risc0-zkvm-elf/release/registry \
     methods/guest/target/riscv32im-risc0-zkvm-elf/docker/registry.bin

   lgs spel -- program-id \
     methods/guest/target/riscv32im-risc0-zkvm-elf/docker/registry.bin \
     --format hex
   ```

   `MAKE_BIN` is a maintainer-supplied path to `make-bin` built from
   `risc0-binfmt` 3.0.5 (the docs only show the local path
   `/home/xps/logos-work/make-bin/target/release/make-bin`; no public install
   command was verifiable). The last command must print
   `4df7d5396b68a16aae6d0f768ef447a0055eb43866827154f71139e486265144`;
   otherwise the measured binary is not the documented deployed binary.

5. **Report cycles with provenance.** Keep this schema in
   `docs/PERFORMANCE.md`:

   | Instruction | user cycles (median / p95) | segments | total cycles (po2) | Iterations | Index length N | Input sizes | ELF SHA-256 | Tool + commit | Notes |
   | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |

   Required rows: `init_index`, `register`, `update`, `mark_verified`, `lookup`,
   `search`. `register` and `update` must not be left as `TBD` once the criterion
   is claimed.

6. **Make the regression reproducible.** Commit the harness, its fixture shape,
   the exact command, and the tool/LEZ/Risc0 versions alongside the numbers.
   Attach the JSON output to the PR or commit it under `artifacts/`.

### 2.3 Cycle acceptance criteria

- The unsupported `cycle_bench --program-id ...` command no longer appears.
- Official LEZ `cycle_bench` output is labelled as a baseline, not as proclaim
  results.
- `register` and `update` cycle rows are populated only from a harness that
  executes the exact program ID above.
- Every number has iteration count, index size, ELF hash, tool version, OS/CPU,
  and Risc0 version.
- If any of that data cannot be produced, the table stays `TBD` and the criterion
  stays unchecked.

---

## 3. Automatic real-sequencer E2E CI

### 3.1 Feasibility finding

A standalone sequencer is the right CI target. LEZ v0.2.4 defines:

```toml
[features]
default = []
standalone = ["sequencer_core/mock"]
```

and its `Justfile` has:

```bash
cargo run --features standalone --release -p sequencer_service -- \
  configs/debug/sequencer_config.json
```

The v0.2.4 sequencer CLI accepts a config path plus `--port`,
`--listen-address`, `--home`, and `--metrics-address`. The debug wallet config
points to `http://127.0.0.1:3040`, and the debug sequencer config contains
genesis-funded accounts. `logos-scaffold` documents that `setup` builds the
standalone sequencer, wallet, and `spel` binaries locally, and that
`test-node start` spawns isolated short-lived sequencers with `--port N`.

Therefore GitHub Actions can run a **real standalone sequencer** on `ubuntu-24.04`
without Docker Bedrock or a public endpoint, provided the pinned LEZ/wallet/
spel sources build there. This is a source-level feasibility conclusion; the
actual GHA run is still required as proof.

No public prebuilt sequencer binary or public container was confirmed from the
public GitHub data checked: LEZ releases showed no usable assets and the GHCR
org package listing returned `401`. The plan therefore builds from the pinned
source. There is also no publicly available always-on sequencer endpoint, which
is exactly why `docs/CI.md` says the stricter E2E criterion is pending.

### 3.2 Port and wallet decision

There is a real port mismatch that maintainers must resolve before merging:

- `proclaim/scaffold.toml` and `LP0023.md` mention localnet port `3041`.
- LEZ debug wallet config and the v0.2.4 sequencer default use `3040`.
- `logos-scaffold` docs say `localnet` waits for `127.0.0.1:3040`.

This plan uses an ephemeral `lgs test-node start --port 3040` and rewrites the
project wallet config to `http://127.0.0.1:3040`. That matches authoritative LEZ
config and avoids relying on a long-lived localnet. If maintainers prefer 3041,
the wallet config, browser, API, and scaffold settings must be changed together
and verified in one proof run.

Use disposable public LEZ debug accounts, never real funds:

```bash
AUTHOR=Public/CbgR6tj5kWx5oziiFptM7jMvrQeYY3Mzaao6ciuhSr2r
AUTHOR_KEY=7f273098f25b71e6c005a9519f2678da8d1c7f01f6a27778e2d9948abdf901fb
VERIFIER=Public/2RHZhw9h534Zr3eq2RGhQete2Hh667foECzXPmSkGni2
VERIFIER_KEY=f434f8741720014586ae43356d2aec6257da086222f604ddb75d69733b86fc4c
```

These keys and account IDs are public LEZ v0.2.4 debug values from the LEZ
`Justfile` and debug sequencer config. Maintainers must confirm they are
acceptable as no-value CI accounts; otherwise they must supply an equivalent
disposable wallet home.

### 3.3 Workflow shape

Replace the manual `e2e` gate in `.github/workflows/ci.yml` with an always-on job.
Use no branch filter so it runs on every push and PR, and therefore on the
default branch:

```yaml
name: CI

on:
  push:
  pull_request:
  workflow_dispatch:

env:
  CARGO_TERM_COLOR: always
  CARGO_BUILD_JOBS: 2
  RUSTFLAGS: '--cfg getrandom_backend="custom"'
  RISC0_DEV_MODE: '1'

jobs:
  e2e:
    name: E2E — real standalone sequencer
    runs-on: ubuntu-24.04
    timeout-minutes: 60
    steps:
      - uses: actions/checkout@v4
      - name: Install native dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            build-essential clang libclang-dev libssl-dev pkg-config \
            pkgconf libpcsclite-dev lsof curl jq
      - name: Install Rust/Risc0/scaffold tooling
        run: |
          curl -L https://risc0.com/install | bash
          echo "$HOME/.risc0/bin" >> "$GITHUB_PATH"
          export PATH="$HOME/.risc0/bin:$PATH"
          rzup install cargo-risczero 3.0.5
          cargo install logos-scaffold
      - name: Build pinned guest, IDL, wallet, spel, sequencer
        run: |
          export PATH="$HOME/.risc0/bin:$PATH"
          lgs setup
          lgs build
          lgs test-node prepare
          lgs doctor --json
      - name: Run full lifecycle against a fresh standalone sequencer
        run: bash scripts/e2e-real-sequencer.sh
      - name: Upload diagnostics
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: e2e-diagnostics
          path: |
            .scaffold/logs/
            .scaffold/test-nodes/
            e2e-artifacts/
          if-no-files-found: ignore
```

Notes:

- `rzup install cargo-risczero 3.0.5` and `cargo install logos-scaffold` come
  from the authoritative scaffold docs. `lgs build` for a SPEL project uses the
  Docker guest builder; `ubuntu-24.04` has a Docker daemon available.
- `RISC0_DEV_MODE=1` is documented by LEZ as disabling proof verification for
  fast iteration. If the prize wording requires full proof verification, remove
  it in a second proving-enabled job or gate; do not claim proof verification
  while using dev mode.
- The workflow must not depend on `E2E_WALLET_HOME_B64` or any private secret if
  the disposable public debug accounts are used.

### 3.4 E2E script: `scripts/e2e-real-sequencer.sh`

The following commands are the concrete lifecycle to add as a new script. They
use only documented commands and flags.

```bash
#!/usr/bin/env bash
set -euo pipefail

PID=4df7d5396b68a16aae6d0f768ef447a0055eb43866827154f71139e486265144
BASE58_PID=6FMaVQ6CmbAVNHgwsmmeBe5kiiKkQRFtqu4GUQ2vAf2T
INDEX_PDA=9mr5hePKo24ZHNztzVpuRND7PeAsR17yQ2M5HHPKcbw3
AUTHOR=Public/CbgR6tj5kWx5oziiFptM7jMvrQeYY3Mzaao6ciuhSr2r
AUTHOR_KEY=7f273098f25b71e6c005a9519f2678da8d1c7f01f6a27778e2d9948abdf901fb
VERIFIER=Public/2RHZhw9h534Zr3eq2RGhQete2Hh667foECzXPmSkGni2
VERIFIER_KEY=f434f8741720014586ae43356d2aec6257da086222f604ddb75d69733b86fc4c
BIN=methods/guest/target/riscv32im-risc0-zkvm-elf/docker/registry.bin
IDL_CID=${E2E_IDL_CID:?maintainer must supply E2E_IDL_CID}
CODE_CID=${E2E_CODE_CID:?maintainer must supply E2E_CODE_CID}
EVIDENCE_CID=${E2E_EVIDENCE_CID:?maintainer must supply E2E_EVIDENCE_CID}
COMMIT=$(git rev-parse HEAD)
BUILD_CONFIG='guest=local,risc0=3.0.5,target=riscv32im-risc0-zkvm-elf'
TIMESTAMP=$(date +%s)
mkdir -p e2e-artifacts

# 1. Assert the built artifact is the documented deployed program.
BUILT_ID=$(lgs spel -- program-id "$BIN" --format hex | tr -d '\r' | tail -n 1)
BUILT_ID=${BUILT_ID#0x}
test "$BUILT_ID" = "$PID"

# 2. Start a fresh real standalone sequencer on the LEZ debug port.
NODE_JSON=$(lgs test-node start --port 3040 --json | tee e2e-artifacts/node.json)
NODE_REF=$(printf '%s' "$NODE_JSON" | jq -r '.id // .state_dir')
test -n "$NODE_REF" && test "$NODE_REF" != null
trap 'lgs test-node stop --node "$NODE_REF" || true' EXIT

lgs test-node status --node "$NODE_REF" --json | tee e2e-artifacts/node-status.json

# 3. Point the project wallet at that sequencer and import disposable accounts.
test -f .scaffold/wallet/wallet_config.json
jq '.sequencers[0].sequencer_addr = "http://127.0.0.1:3040"' \
  .scaffold/wallet/wallet_config.json > e2e-artifacts/wallet_config.json
mv e2e-artifacts/wallet_config.json .scaffold/wallet/wallet_config.json

lgs wallet -- account import public --private-key "$AUTHOR_KEY"
lgs wallet -- vault claim --account-id "$AUTHOR" --amount 10000
lgs wallet -- account import public --private-key "$VERIFIER_KEY"
lgs wallet -- vault claim --account-id "$VERIFIER" --amount 20000
lgs wallet -- account list
lgs wallet default set "$AUTHOR"

# 4. Deploy → init-index → register → update → search → mark-verified.
lgs wallet -- deploy-program "$BIN"
lgs spel -- init-index --authority "$AUTHOR"

lgs spel -- register --key "$PID" \
  --name 'LP-0023 CI Registry' --version '0.3.0' \
  --description 'Automatic E2E registry entry' \
  --tags 'registry,spel,lez,search,verified' \
  --idl-cid "$IDL_CID" --code-cid "$CODE_CID" \
  --commit "$COMMIT" --build-config "$BUILD_CONFIG" \
  --timestamp "$TIMESTAMP" --deployer-claimed true \
  --author "$AUTHOR"

lgs spel -- update --key "$PID" \
  --version '0.3.1' \
  --description 'Automatic E2E registry entry after update' \
  --tags 'registry,spel,lez,search,verified,updated' \
  --idl-cid "$IDL_CID" --code-cid "$CODE_CID" \
  --commit "$COMMIT" --build-config "$BUILD_CONFIG" \
  --author "$AUTHOR"

lgs spel -- search | tee e2e-artifacts/spel-search.txt
grep -F 'LP-0023 CI Registry' e2e-artifacts/spel-search.txt

ENTRY_PDA=$(lgs spel -- pda entry --key "$PID" | tr -d '\r' | tail -n 1)
INDEX_PDA_OUT=$(lgs spel -- pda index | tr -d '\r' | tail -n 1)
test "$INDEX_PDA_OUT" = "$INDEX_PDA"

lgs spel -- inspect "$ENTRY_PDA" --type RegistryEntry | tee e2e-artifacts/entry-before-verify.txt
lgs spel -- inspect "$INDEX_PDA" --type RegistryIndex | tee e2e-artifacts/index.txt

lgs spel -- mark-verified --key "$PID" \
  --method 'reproducible-rebuild:program-id' \
  --evidence-cid "$EVIDENCE_CID" --verifier "$VERIFIER"

lgs spel -- inspect "$ENTRY_PDA" --type RegistryEntry | tee e2e-artifacts/entry-after-verify.txt
grep -F "$EVIDENCE_CID" e2e-artifacts/entry-after-verify.txt

# 5. Browser read, including the verified-only path.
cargo build -p registry-examples --bin registry_browser
./target/debug/registry_browser --program-id "$PID" --index-only \
  | tee e2e-artifacts/browser-index.txt
grep -F "$INDEX_PDA" e2e-artifacts/browser-index.txt || true

./target/debug/registry_browser --program-id "$PID" \
  --name 'LP-0023 CI Registry' --tag registry --verified --json \
  | tee e2e-artifacts/browser-search.json \
  | jq -e 'length >= 1'

# 6. HTTP API read, backed by the same on-chain sequencer state.
export LEE_WALLET_HOME_DIR="$PWD/.scaffold/wallet"
export REGISTRY_PROGRAM_ID="$PID"
cargo run -p registry-api --bin registry-api > e2e-artifacts/api.log 2>&1 &
API_PID=$!
trap 'kill "$API_PID" 2>/dev/null || true; lgs test-node stop --node "$NODE_REF" || true' EXIT

for _ in $(seq 1 30); do
  if curl -fsS http://127.0.0.1:3030/health > e2e-artifacts/health.json; then
    break
  fi
  sleep 1
done
jq -e '.status == "ok"' e2e-artifacts/health.json

curl -fsS http://127.0.0.1:3030/index | tee e2e-artifacts/api-index.json \
  | jq -e --arg p "$BASE58_PID" 'index($p) != null'
curl -fsS http://127.0.0.1:3030/entries | tee e2e-artifacts/api-entries.json \
  | jq -e 'length >= 1'
curl -fsSG http://127.0.0.1:3030/search \
  --data-urlencode 'name=LP-0023 CI Registry' \
  --data-urlencode 'tag=registry' \
  --data-urlencode 'verified=true' \
  --data-urlencode 'limit=10' \
  | tee e2e-artifacts/api-search.json \
  | jq -e 'length >= 1'
curl -fsS "http://127.0.0.1:3030/entry/$BASE58_PID" \
  | tee e2e-artifacts/api-entry.json \
  | jq -e '.source_verified == true'
```

Validation notes for the first proof run:

- `lgs test-node start --json` documents `state_dir` and connection fields; the
  fallback `.id // .state_dir` above relies only on documented fields. If the
  actual JSON uses a different identifier, adjust the `NODE_REF` extraction to
  the real output rather than guessing a new flag.
- The `grep` against `spel inspect` output checks human-readable fields whose
  exact formatting was not verifiable from public source alone. Replace with a
  JSON assertion if the pinned `spel` supports machine-readable inspect output.
- The browser `--program-id`, `--name`, `--tag`, `--verified`, `--json`, and
  `--index-only` flags are confirmed in
  `examples/src/bin/registry_browser.rs`.
- API routes and query parameters (`/health`, `/index`, `/entries`, `/search`,
  `/entry/:program_id`; `name`, `tag`, `verified`, `limit`) are confirmed in
  `registry-api/src/main.rs`.

### 3.5 Update repository docs as part of implementation

As part of the repo change that adds the workflow, also update
`docs/CI.md`:

- Remove the statement that the stricter E2E criterion is still pending once the
  workflow is green on the default branch.
- State that the job starts a fresh standalone sequencer on every push/PR, uses
  disposable public debug accounts, and performs deploy → init-index →
  register → update → search → mark-verified → browser/API reads.
- State explicitly whether `RISC0_DEV_MODE=1` is used, so no one mistakes the
  test for full proof verification.

### 3.6 E2E acceptance criteria

- The job runs on `push` with no branch filter and is required/green on the
  default branch.
- It starts a real LEZ standalone sequencer in the job; no public endpoint or
  private wallet secret is required.
- The deployed program ID is asserted to be
  `4df7d5396b68a16aae6d0f768ef447a0055eb43866827154f71139e486265144`.
- Every lifecycle step above succeeds in order, including `update` and
  independent `mark-verified` by a different account.
- `registry_browser` and all five API routes read the freshly written state,
  including the verified-only filter.
- Diagnostics are uploaded on failure.
- The criterion is not checked until at least one successful default-branch GHA
  run is linked in the solution document.

---

## 4. Risks and unverifiable public information

These items cannot be honestly asserted from the public information reviewed:

1. **No public always-on sequencer endpoint** was found; current `docs/CI.md`
   says the same. The standalone sequencer removes this dependency but must be
   proven in GHA.
2. **No public prebuilt sequencer binary/container was confirmed.** Use pinned
   source builds. Do not document a container image or release asset that was
   not verified.
3. **The exact `make-bin`/`risc0-binfmt` install source was not publicly
   verified.** The only evidence is a local path in `LP0023.md`. Maintainers
   must supply a public source, pinned container, or committed artifact.
4. **On-chain values in `LP0023.md` (program ID, PDAs, CIDs, tx hashes,
   author/verifier) could not be independently verified** without a live
   endpoint. Treat them as maintainer-provided facts until the E2E job asserts
   them.
5. **Logos Storage CIDs are not fetchable over HTTP**; the docs describe a QtRO
   socket workflow. The E2E job should use maintainer-supplied CIDs as metadata
   strings, not claim content availability.
6. **The GHA run itself has not been executed.** Build time, Docker availability,
   LEZ source build success on `ubuntu-24.04`, and wallet CLI behavior are
   source-backed expectations, not observed results.
7. **Port discrepancy (3040 vs 3041)** must be resolved and tested. Mixing
   scaffold settings, wallet config, and browser/API assumptions will create a
   false failure.
8. **LEZ `dev` docs and v0.2.4 differ** in sample programs and criterion bench
   names. Pin the ref used for any reported number; do not mix tables.
9. **`RISC0_DEV_MODE=1` weakens proof claims.** It is acceptable for fast E2E
   lifecycle testing only if documented honestly.
10. **SPEL/spec is changing.** Open PRs in the solution doc indicate possible
    future spec changes; re-check immediately before claiming the prize.

### Data maintainers must supply or confirm

- `E2E_IDL_CID`, `E2E_CODE_CID`, and `E2E_EVIDENCE_CID` for CI metadata.
- Confirmation that the public LEZ v0.2.4 debug accounts/keys are acceptable as
  disposable CI identities, or a replacement no-value wallet home.
- The chosen sequencer port (this plan recommends 3040) and any scaffold config
  changes needed to make it consistent.
- A reproducible guest artifact path/source for `make-bin`, or a committed
  prebuilt `registry.bin` whose ID equals the target program ID.
- Host CPU/OS and Risc0 versions for cycle reporting.
- The proclaim-specific cycle harness and its fixture definitions for
  `register`/`update`.
- Whether a full-proving E2E job is required in addition to the dev-mode job.

---

## 5. Recommended execution order

1. Fix `docs/PERFORMANCE.md`: delete the unsupported cycle command, label LEZ
   `cycle_bench` as a built-in baseline, keep `TBD`.
2. Add the standalone-sequencer E2E workflow and script above; run it on a PR
   until green.
3. Resolve the 3040/3041 port discrepancy in one proof run.
4. Update `docs/CI.md` only after a green default-branch run; link that run in
   `docs/SOLUTION-LP-0023.md`.
5. Implement the proclaim guest-cycle harness; measure `register` and `update`
   against the exact program ID.
6. Replace the `TBD` cycle rows with provenance-backed measurements and attach
   the JSON artifacts.
7. Re-check the LP-0023 spec and solution PRs before final submission.

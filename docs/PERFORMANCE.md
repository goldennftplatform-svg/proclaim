# Performance and cost notes

This document records the on-chain cost model for the LP-0023 registry and the
procedure for measuring it. **Cycle counts have not been measured yet** — the
table at the end is deliberately left as `TBD` rather than filled with estimates.

## What is on-chain

Only metadata and pointers are stored on-chain. The IDL and the source archive
are content-addressed on Logos Storage and referenced by CID.

| Account | Size | Grows with |
| --- | --- | --- |
| `RegistryIndex` (`registry_index` PDA) | `4 + 32 * N` bytes Borsh (vec length prefix + one 32-byte program ID each) | Number of registered programs |
| `RegistryEntry` (one PDA per program ID) | Fixed layout: `program_id`, `name`, `version`, `author`, `description`, `tags`, `idl_cid`, `code_cid`, `commit`, `build_config`, `timestamp`, `deployer_claimed`, `source_verified`, `verifier`, `verification_method`, `evidence_cid` | Constant per entry (strings are caller-bounded) |

Because an entry PDA is derived directly from the program ID (`pda = arg("key")`),
a `lookup` never scans the index. `search` reads one index account and then
fetches entry accounts; the cost is linear in the number of entries and needs no
indexer service.

## Account access per instruction

Derived from the SPEL annotations in `methods/guest/src/bin/registry.rs`:

| Instruction | Accounts written | Accounts read | Notes |
| --- | --- | --- | --- |
| `init_index` | 1 (`registry_index`, init) | 1 signer | Called once |
| `register` | 2 (entry init + index append) | 1 signer | Index append makes cost grow with `N` |
| `update` | 1 (entry) | 1 signer | Bounded; clears verification on source-field change |
| `mark_verified` | 1 (entry) | 1 signer | Bounded |
| `lookup` | 0 | 1 (entry PDA) | Read-only |
| `search` | 0 | 1 (index PDA) | Read-only; host fetches entries |

The two read-only instructions do not mutate accounts, so their execution cost is
independent of registry size.

## How to measure cycle counts

No figures are claimed yet. Two things are required: an **LEZ baseline** with the
official tool, and a **proclaim-specific harness** for this program's
`register`/`update` instructions. Both are described below; neither has produced
a measured proclaim number, so the table stays `TBD`.

### 1. Official LEZ baseline (built-in guests only — NOT proclaim cycles)

`cycle_bench` ships in the LEZ source tree. An earlier draft of this document
used a `cycle_bench --program-id … --instruction … --iterations …` command; that
command **does not exist**. The authoritative CLI (LEZ `dev`,
`tools/cycle_bench/src/main.rs`) accepts only `--prove`, `--ppe` and
`--exec-iters`, benchmarks a fixed list of built-in guests (`clock`, `fee`,
`scripted`, `system_upgrader`, `sequencer_stake`), and cannot load an arbitrary
deployed program ID. The documented invocations, for the v0.2.4 tag this project
pins, are:

```bash
git clone --branch v0.2.4 --depth 1 https://github.com/logos-blockchain/logos-execution-zone.git
cd logos-execution-zone
cargo run --release -p cycle_bench -- --exec-iters 50
cargo run --release -p cycle_bench --features prove -- --prove
cargo run --release -p cycle_bench --features ppe -- --prove --ppe
cargo bench -p cycle_bench --features ppe --bench verify
```

This writes `target/cycle_bench.json` and (version-dependently)
`target/criterion/`. It reports `user_cycles`, `segments`, `exec_ms`, and with
proving `total_cycles`, `prove_ms` and `proof_bytes`. These numbers are LEZ
built-ins and are useful only as a baseline; they are **not** the registry's
`register`/`update` cycles.

### 2. Proclaim-specific harness (not yet implemented)

Because `cycle_bench` cannot load program ID
`4df7d5396b68a16aae6d0f768ef447a0055eb43866827154f71139e486265144`, measuring
the registry requires a small harness that executes the exact deployed R0BF
artifact with the same fixtures as the on-chain transactions. The required
shape is documented in [`docs/CYCLES-E2E-PLAN.md`](CYCLES-E2E-PLAN.md): load
`methods/guest/target/riscv32im-risc0-zkvm-elf/docker/registry.bin`, assert the
derived program ID, build root/entry/index fixtures for each instruction,
execute with `risc0_zkvm::default_executor().execute(env, elf)`, discard a
warm-up run, and report median/p95 `cycles()` plus `segments.len()` against a
machine-readable artifact. Rebuild the ELF from published source first:

```bash
source ~/.cargo/env
export PATH="$HOME/.risc0/bin:$PATH"
export CARGO_BUILD_JOBS=2
export RUSTFLAGS='--cfg getrandom_backend="custom"'
cargo +risc0 build --release --manifest-path methods/guest/Cargo.toml \
  --target riscv32im-risc0-zkvm-elf
```

Until that harness exists and its output is committed here, no registry cycle
count is claimed. Do not infer cycles from the account-access table above.

## Measured cycle counts

> **Pending.** Do not infer figures from the account counts above.

| Instruction | Median cycles | p95 cycles | Iterations | Tool version | Notes |
| --- | --- | --- | --- | --- | --- |
| `init_index` | TBD | TBD | TBD | TBD | One-time setup |
| `register` | TBD | TBD | TBD | TBD | Depends on index length `N` |
| `update` | TBD | TBD | TBD | TBD | |
| `mark_verified` | TBD | TBD | TBD | TBD | |
| `lookup` | TBD | TBD | TBD | TBD | Read-only |
| `search` | TBD | TBD | TBD | TBD | Read-only, index account only |

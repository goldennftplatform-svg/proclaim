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

The LEZ `cycle_bench` tool was not available in the development environment (no
`cycle_bench` binary and no LEZ source tree locally), so no measured figures are
claimed. To fill the table below:

1. Install the LEZ toolchain that provides `cycle_bench` (it ships with the LEZ
   developer toolchain / `lgs` plugin set; check `lgs --help` after updating).
2. Build the guest from the published source so the measured ELF is the deployed
   one:

   ```bash
   source ~/.cargo/env
   export RUSTFLAGS='--cfg getrandom_backend="custom"'
   CARGO_BUILD_JOBS=2 cargo build --manifest-path methods/guest/Cargo.toml --release --target risc0
   ```

3. Run the bench for each instruction against the local sequencer, using the
   same account PDAs as `LP0023.md`:

   ```bash
   cycle_bench --program-id 4df7d5396b68a16aae6d0f768ef447a0055eb43866827154f71139e486265144 \
     --instruction register --iterations 100
   ```

4. Record the median and p95 cycles, the account state size used, and the tool
   version, then commit the numbers here.

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

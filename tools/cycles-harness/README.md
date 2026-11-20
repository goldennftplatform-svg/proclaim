# proclaim-specific cycle-count harness (LP-0023)

This crate measures **real, local Risc0 execution cycles** for the exact
proclaim guest artifact whose image ID is the deployed program ID:

```
4df7d5396b68a16aae6d0f768ef447a0055eb43866827154f71139e486265144
```

It does **not** edit the proclaim repository, commit, push, or use tokens.
It is a standalone host-side harness under `staged/cycles-harness/`.

## Why a custom harness is required

The authoritative LEZ `tools/cycle_bench/src/main.rs` only benchmarks its
built-in guests (`clock`, `fee`, `scripted`, `system_upgrader`,
`sequencer_stake`). Its only flags are `--prove`, `--ppe`, and `--exec-iters`;
it has no `--program-id`, `--instruction`, or `--iterations` flag. It cannot
load the deployed proclaim program ID. Therefore LEZ `cycle_bench` numbers are
built-in baselines, **not** proclaim `register`/`update` cycles.

## Artifact and program-ID assertion

The harness defaults to the already-built R0BF artifact:

```
/home/xps/logos-work/registry/methods/guest/target/riscv32im-risc0-zkvm-elf/docker/registry.bin
```

Before executing anything, it asserts both:

- SHA-256 of the R0BF bytes equals
  `88716a6ba8c67e9623f3b2d5acf1d9c758ce071dbce391f1d9084628d72506c5`;
- `risc0_zkvm::compute_image_id()` on those bytes, converted to little-endian
  hex exactly as `spel-cli/src/inspect.rs` does, equals the deployed program ID
  above.

If either assertion fails, the harness exits with an error and writes no
report.

Independently verified local ground truth:

```bash
lgs spel -- program-id \
  /home/xps/logos-work/registry/methods/guest/target/riscv32im-risc0-zkvm-elf/docker/registry.bin \
  --format hex
# 4df7d5396b68a16aae6d0f768ef447a0055eb43866827154f71139e486265144
```

## Verified API facts (from local source, not guessed)

- `risc0-zkvm 3.0.5` `default_executor().execute(env, elf)` returns
  `SessionInfo`; `SessionInfo::cycles()` sums segment user cycles and
  `SessionInfo::segments.len()` is the segment count.
  (`risc0-zkvm-3.0.5/src/host/api/mod.rs`, `host/client/prove/mod.rs`)
- `ExecutorImpl::from_elf` calls `ProgramBinary::decode(elf)`, so the bytes
  passed to `execute()` must be the **R0BF-wrapped `.bin`**, not the raw ELF
  `release/registry`. (`risc0-zkvm-3.0.5/src/host/server/exec/executor.rs`)
- The guest reads four env values in order (`lee_core::read_lee_inputs`):
  `ProgramId`, `Option<ProgramId>`, `Vec<AccountWithMetadata>`, `Vec<u32>`.
- `ProgramId` is `[u32; 8]`, `InstructionData` is `Vec<u32>`,
  `AccountId::for_public_pda(program_id, &PdaSeed::new(seed))` derives PDAs.
  (`lee_core 0.1.0` / tag `v0.2.4`)
- SPEL generates one `Instruction` enum variant per `#[instruction]`, in source
  order. Unit variants serialize as just their `u32` index; struct variants as
  index followed by fields. (`spel-framework-macros`, `risc0-zkvm` Serializer)
- Without the `prove` feature, `default_executor()` uses `ExternalProver` and
  shells out to `r0vm`, which is **not installed** here. With `prove`, it uses
  the in-process `LocalProver`. This crate enables `prove` with `bonsai`
  disabled.

## Fixture shape

All fixtures are synthetic; no real on-chain state or private keys are used.
They mirror the six instructions in
`/home/xps/logos-work/registry/methods/guest/src/bin/registry.rs`:

| Instruction | Pre-states | Notes |
| --- | --- | --- |
| `init_index` | index PDA (`Account::default()`), signer authority | `init` + `pda=literal("registry_index")` |
| `register` | init entry PDA, index PDA with `N` IDs, signer author | `pda=arg("key")` for entry |
| `update` | existing entry, signer author | source-identifying fields change |
| `mark_verified` | existing entry, independent signer verifier | author cannot verify own entry |
| `lookup` | existing entry | read-only |
| `search` | index PDA with `N` IDs | read-only |

All six guest handlers call `SpelOutput::execute(post_states, vec![])`, so
there are **no chained-call transitions**; the measured root execution is the
complete guest transition set for these fixtures.

## Build and run

The local `risc0-zkvm 3.0.5` release is yanked on crates.io. This crate ships a
`Cargo.lock` copied from the working registry workspace, which legitimately
pins `risc0-zkvm 3.0.5` and the same `lee_core` git revision. Use that lock.

The task's `RUSTFLAGS='--cfg getrandom_backend="custom"'` requires a host
symbol. The harness defines `__getrandom_v03_custom` in `src/main.rs` and reads
`/dev/urandom`. Building with `--target x86_64-unknown-linux-gnu` keeps
`RUSTFLAGS` off host build scripts (otherwise `risc0-sys` build scripts fail to
link with `undefined symbol: __getrandom_v03_custom`).

```bash
source ~/.cargo/env
export RUSTFLAGS='--cfg getrandom_backend="custom"'
export CARGO_BUILD_JOBS=2
export PATH="$HOME/.risc0/bin:$PATH"

cd /mnt/c/Users/XPS/Downloads/GFD/staged/cycles-harness
cargo build --target x86_64-unknown-linux-gnu --locked

./target/x86_64-unknown-linux-gnu/debug/cycles-harness \
  --elf /home/xps/logos-work/registry/methods/guest/target/riscv32im-risc0-zkvm-elf/docker/registry.bin \
  --iterations 20 \
  --index-len 8 \
  --output /mnt/c/Users/XPS/Downloads/GFD/staged/cycles-harness/cycles-report.json
```

Optional flags: `--elf PATH`, `--iterations N` (>= 2), `--index-len N`,
`--output PATH`.

## Measurement method

- One warm-up execution is discarded per instruction.
- Remaining executions report min ("best"), mean, population stdev, median,
  p95 (nearest-rank `sorted[ceil(0.95*n)-1]`), and max.
- Cycles are `SessionInfo::cycles()` (user cycles, no proof/po2 padding).
- Segments are `SessionInfo::segments.len()`.
- `total_cycles` (po2/proving) is **not** measured by this execution-only run
  and is reported as `null` rather than fabricated.
- Index length `N` defaults to 8; instruction word counts are recorded per
  result. String/CID fixture lengths are fixed in `src/main.rs`.
- The host binary is built in the `dev` profile for speed. Guest cycles are
  VM-deterministic and do not depend on the host build profile.
- `cycles-report.json` records OS, kernel, CPU model, rustc version, guest repo
  commit, ELF hash, and executor call shape for provenance.

## Results and outputs

Measured medians (`iterations=20`, `index-len=8`, 1 warm-up discarded):
`init_index` 86,937; `register` 319,445; `update` 226,298;
`mark_verified` 230,009; `lookup` 169,398; `search` 164,656 user cycles.
All samples were identical (stdev 0) and used 1 segment.

- `cycles-report.json` — machine-readable provenance and all statistics.
- `run-output.txt` — verbatim captured stdout/stderr of the successful run.
- `build-output.txt` — verbatim build log.
- `../CYCLES-HARNESS-RESULT.md` — one-level-up closeout with the full table.

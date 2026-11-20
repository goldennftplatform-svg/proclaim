# LP-0023 cycle harness result

Status: **SUCCESS — real guest cycle numbers obtained** on 2026-11-19/20 UTC.
No proclaim repository edits, commits, pushes, tokens, or PRs were made.

## 6-line summary

1. Created the standalone harness at `tools/cycles-harness/` with `Cargo.toml` (own `[workspace]` marker so it is isolated from the root workspace), `Cargo.lock`, `src/main.rs`, `README.md`, `cycles-report.json`, `run-output.txt` and `build-output.txt`.
2. The harness asserts the R0BF ELF SHA-256 `88716a6ba8c67e9623f3b2d5acf1d9c758ce071dbce391f1d9084628d72506c5` and computed image ID `4df7d5396b68a16aae6d0f768ef447a0055eb43866827154f71139e486265144` before measuring.
3. Real user-cycle medians (20 samples, 1 warm-up discarded, `N=8`, index append path): `register` **319,445**, `update` **226,298**, `init_index` **86,937**, `mark_verified` **230,009**, `lookup` **169,398**, `search` **164,656**.
4. All six executions used 1 segment; every instruction had identical cycles across samples (stdev 0), which is expected for deterministic zkVM execution.
5. Method: `risc0_zkvm::default_executor().execute(env, &r0bf_bytes)` in-process (`prove` feature, no `r0vm` needed); `SessionInfo::cycles()` and `segments.len()` are the cycle/segment sources.
6. No technical blocker remains; only deliberate limits are synthetic fixtures, no po2/proving totals (reported `null`, not fabricated), and no chained calls because all six guest handlers return `SpelOutput::execute(post_states, vec![])`.

## Files created

Under `tools/cycles-harness/`:

- `Cargo.toml` — pinned `risc0-zkvm = 3.0.5` (`std,prove`; Bonsai disabled), `lee_core` git tag `v0.2.4`.
- `Cargo.lock` — copied from the working registry checkout, then resolved by Cargo; required because `risc0-zkvm 3.0.5` is yanked on crates.io.
- `src/main.rs` — ELF/hash/program-ID assertions, six fixtures, executor, statistics, JSON and Markdown output.
- `README.md` — API provenance, fixture table, build/run commands, and measurement semantics.
- `cycles-report.json` — machine-readable report with all measurements and provenance.
- `run-output.txt` — verbatim stdout/stderr from the successful run.
- `build-output.txt` — verbatim build log (extra provenance artifact).

One-level-up report:

- `docs/CYCLES-HARNESS-RESULT.md` — this file.

No `BLOCKERS.md` was created because the harness ran successfully.

## Exact command used

```bash
source ~/.cargo/env
export RUSTFLAGS='--cfg getrandom_backend="custom"'
export CARGO_BUILD_JOBS=2
export CARGO_TERM_COLOR=never
export PATH="$HOME/.risc0/bin:$PATH"

cd tools/cycles-harness
cargo build --target x86_64-unknown-linux-gnu --locked

./target/x86_64-unknown-linux-gnu/debug/cycles-harness \
  --elf /home/xps/logos-work/registry/methods/guest/target/riscv32im-risc0-zkvm-elf/docker/registry.bin \
  --iterations 20 \
  --index-len 8 \
  --output tools/cycles-harness/cycles-report.json
```

`--target x86_64-unknown-linux-gnu` is intentional: with the task's global
`RUSTFLAGS='--cfg getrandom_backend="custom"'`, host build scripts would
otherwise fail to link `undefined symbol: __getrandom_v03_custom`. The harness
defines that symbol for its own process; passing `--target` keeps `RUSTFLAGS`
off host build-script units.

## Measured table

| Instruction | user cycles best / median / p95 | mean ± stdev | segments | total cycles (po2) | iterations | index N | instruction words | ELF SHA-256 | tool + commit | notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `init_index` | 86,937 / **86,937** / 86,937 | 86,937.0 ± 0.0 | 1 | not measured | 20 | 8 | 1 | `88716a6b…2506c5` | risc0-zkvm 3.0.5 / guest `383afbee…` | empty index PDA + signer authority |
| `register` | 319,445 / **319,445** / 319,445 | 319,445.0 ± 0.0 | 1 | not measured | 20 | 8 | 79 | `88716a6b…2506c5` | risc0-zkvm 3.0.5 / guest `383afbee…` | init entry PDA + 8-entry index + signer author; index excludes key so append path runs |
| `update` | 226,298 / **226,298** / 226,298 | 226,298.0 ± 0.0 | 1 | not measured | 20 | 8 | 72 | `88716a6b…2506c5` | risc0-zkvm 3.0.5 / guest `383afbee…` | existing entry + signer author; source-identifying fields change |
| `mark_verified` | 230,009 / **230,009** / 230,009 | 230,009.0 ± 0.0 | 1 | not measured | 20 | 8 | 49 | `88716a6b…2506c5` | risc0-zkvm 3.0.5 / guest `383afbee…` | existing entry + independent signer verifier |
| `lookup` | 169,398 / **169,398** / 169,398 | 169,398.0 ± 0.0 | 1 | not measured | 20 | 8 | 33 | `88716a6b…2506c5` | risc0-zkvm 3.0.5 / guest `383afbee…` | existing entry PDA, read-only |
| `search` | 164,656 / **164,656** / 164,656 | 164,656.0 ± 0.0 | 1 | not measured | 20 | 8 | 1 | `88716a6b…2506c5` | risc0-zkvm 3.0.5 / guest `383afbee…` | 8-entry index PDA, read-only |

Full SHA-256: `88716a6ba8c67e9623f3b2d5acf1d9c758ce071dbce391f1d9084628d72506c5`.
Guest repo commit: `383afbee77e22ae9a1bcda06cc2ba7624d63c1d8`.
`lee_core` git rev: `47eba256479f6f785acbd138834340703cd03401` (tag `v0.2.4`).

## Provenance and method

- ELF: `/home/xps/logos-work/registry/methods/guest/target/riscv32im-risc0-zkvm-elf/docker/registry.bin` (R0BF-wrapped, 463,236 bytes).
- Assertions passed in order: ELF SHA-256, then `compute_image_id()` little-endian hex.
- Executor: `risc0_zkvm::default_executor()` with `RISC0_EXECUTOR=local` set explicitly; this is the in-process `LocalProver` because the `prove` feature is enabled. `r0vm` is not installed and was not used.
- Execution call: `default_executor().execute(env, &elf_bytes)` — verified from local risc0 source to accept R0BF `ProgramBinary` bytes and return `SessionInfo`.
- Cycle source: `SessionInfo::cycles()`; segment source: `SessionInfo::segments.len()`.
- Statistics: 1 discarded warm-up + 20 samples; median is upper median `sorted[n/2]`; p95 is nearest rank `sorted[ceil(0.95*n)-1]`; stdev is population stdev.
- Inputs: synthetic but shape-identical to the six guest instructions, with Borsh `RegistryEntry`/`RegistryIndex` layouts copied from the guest source. No real keys, on-chain state, or CIDs.
- Environment: WSL2 Linux `6.18.58.1-microsoft-standard-WSL2`, x86_64, Intel i7-7700HQ, rustc 1.100.0. Host binary built in dev profile; guest cycles are VM-deterministic and independent of host optimization.
- The R0BF format is decoded by the `risc0-zkvm 3.0.5` stack; Cargo also compiled transitive `risc0-binfmt 3.0.6`, as recorded in `cycles-report.json`.

## Honest limitations

- `total_cycles` (po2/proving cycles), `prove_ms`, and proof size were not measured. This is an execution-only harness; the JSON reports `total_cycles_po2: null` rather than inventing values.
- Cycle numbers are for fixed synthetic fixture sizes (`N=8`, fixed strings/CIDs). They are valid measurements of the exact guest, but not universal costs for every possible payload.
- The low-level R0BF artifact was already present locally and independently verified via `lgs spel -- program-id ... --format hex`; the harness does not rebuild the guest.
- The previous `cycle_bench --program-id ...` command remains unsupported; these numbers come only from this harness.

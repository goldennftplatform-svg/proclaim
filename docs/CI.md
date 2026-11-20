# Continuous integration

<!-- SPDX-License-Identifier: MIT AND Apache-2.0 -->

`.github/workflows/ci.yml` runs on every push and pull request to `master`, and
can be dispatched manually. All three jobs are always-on; none requires Docker,
repository secrets, or real funds.

## Jobs

| Job | What it verifies |
| --- | --- |
| `rust` | Native build deps (incl. `libpcsclite-dev`), then `cargo fmt --all -- --check`, `cargo clippy` on the host crates with `-D warnings`, `cargo test -p registry_core` (7 tests), and `cargo check` for `registry-api` + `registry-examples`. |
| `js` | `node --test clients/js/test/registry.test.js` (8 tests) on Node 22. |
| `e2e` | Full real-sequencer lifecycle against a fresh `lgs test-node`: deploy → `init-index` → `register` → `update` → `search` → independent `mark-verified`, then browser JSON and five HTTP API routes read the same state. Pinned LEZ v0.2.4 / SPEL v0.7.0; program ID `4df7d539…65144` is asserted. |

The `rust` job installs `libpcsclite-dev` because `pcsc-sys` (a transitive
dependency of the host crates) needs it; earlier runs failed at Clippy with
exit 101 and `Package 'libpcsclite' was not found` before this was added.

## E2E caveats (do not overclaim)

- `RISC0_DEV_MODE=1` is set explicitly. LEZ documents this as skipping proof
  generation / disabling proof verification. This job tests **lifecycle and
  reads**, not proof verification.
- The two account keys used by the lifecycle are **public disposable LEZ
  debug identities** from the upstream `Justfile`, holding no real funds. They
  are not secrets and are only valid on a fresh local test node.
- Guest cycles are measured separately by the standalone harness in
  [`docs/CYCLES-HARNESS-RESULT.md`](CYCLES-HARNESS-RESULT.md); this job does not
  measure or claim po2/proving cycles.
- Diagnostics (`e2e-artifacts/`, node and wallet state) are uploaded on every
  run, including failures.

## Honest status

The implementation is applied and pushed. The criterion “E2E integration tests
against a real sequencer in CI, green on the default branch” stays **unchecked**
in [`docs/SOLUTION-LP-0023.md`](SOLUTION-LP-0023.md) until a complete
default-branch run is green and linked here. Proof run:
`https://github.com/goldennftplatform-svg/proclaim/actions/runs/44666188261`
(status to be confirmed — do not treat this paragraph as a green claim).

# Continuous integration

`.github/workflows/ci.yml` runs on pushes to `master`, on pull requests, and can
be dispatched manually.

## Always-on jobs (green on the default branch)

| Job | What it verifies |
| --- | --- |
| `rust` | `cargo fmt --all -- --check`, `cargo clippy` on the host crates with `-D warnings`, `cargo test -p registry_core` (7 tests), and `cargo check` for `registry-api` + `registry-examples`. |
| `js` | `node --test clients/js/test/registry.test.js` (8 tests) on Node 22. |

Both jobs are dependency-light: no Docker, no Qt, no sequencer, no private keys.
The guest build (`cargo risczero build`) is intentionally not part of CI because
it needs the RISC Zero toolchain and previously required Docker in this
environment; reproducible guest builds are covered by
[`scripts/verify-source.sh`](../scripts/verify-source.sh).

## Manual E2E job

The `e2e` job is gated behind `workflow_dispatch` with the `run_e2e` input. It
reads the registry index from a **real sequencer** with `registry_browser
--index-only`. It is not enabled on push/PR events because:

1. there is no publicly available always-on sequencer endpoint for this workflow,
2. it needs wallet configuration, and
3. the registry is currently deployed only to the local standalone sequencer.

To enable it:

- Create a **dedicated, disposable wallet with no real funds** (never commit or
  reuse a keystore with value).
- Create a gzipped tarball of that wallet home directory (it must contain
  `wallet_config.json` and, if needed, `storage.json`), base64-encode it, and
  store it as the repository secret `E2E_WALLET_HOME_B64`.
- Point that wallet at the chosen sequencer via its config.
- Run the workflow manually with `run_e2e` checked.

### Honest status

This satisfies the *automated host-side checks in CI* part of the Supportability
criteria. The stricter wording — "E2E integration tests against a real
sequencer in CI, green on the default branch" — is **still pending**: the E2E
job is manual and only performs a read smoke test, not the full
deploy → register → verify lifecycle. It is therefore left unchecked in
`docs/SOLUTION-LP-0023.md` until it runs automatically on every push.

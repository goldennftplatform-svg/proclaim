<!-- SPDX-License-Identifier: MIT AND Apache-2.0 -->

# Registry E2E lifecycle: failing → passing

**Status: green** on the default branch. Run
[44792441611](https://github.com/goldennftplatform-svg/proclaim/actions/runs/44792441611)
at head `1015a8a`. All three jobs pass, including the full E2E lifecycle against
a real standalone sequencer.

This document records the two real bugs found while getting here, the fixes, and
the exact passing evidence. It is not a prize claim and not a deployment claim.

## What the green run actually does

`scripts/e2e-real-sequencer.sh` starts a fresh LEZ standalone sequencer, imports
two disposable debug accounts, deploys the locally built registry guest, then runs:

1. `init-index` and `register` an entry (`LP-0023 CI Registry`)
2. `update` it to version `0.3.1`
3. `mark-verified` with a reproducible-rebuild method and evidence CID
4. `search` through the guest CLI
5. Browser reads (`--index-only` and filtered `--search`) from the same state
6. Read-only HTTP API: `/health`, `/index`, `/entries`, `/search`, `/entry/:id`

All steps passed. From the retained `e2e-diagnostics` artifact
([14660455210](https://github.com/goldennftplatform-svg/proclaim/actions/artifacts/14660455210)):

- `api.log`: `registry program id : 0x5f80583a57b91af90f5e30d3420379ca4e4f7abeecb2c975b6063cfcd944d733`
- `api-index.json`: `["7RoEv9jrMPcjiPfouYKcrxFdjiMB5Yc9hNXRBE3x6BPQ"]`
- `api-entries.json`: 1 entry; `api-search.json`: 1 match
- `api-entry.json`: `name = LP-0023 CI Registry`, `version = 0.3.1`,
  `source_verified = true`, `evidence_cid = zDv*8iXZ6U`,
  `verification_method = reproducible-rebuild:program-id`,
  `commit = 1015a8a77a690b91d16813f2a8da578b0a326e64`

## Bug 1 — search assertion read the wrong output

**Failing:** run `44695605569` at `6c59b9f`. Register, update and search
transactions were all confirmed on the sequencer, but the script grepped the
`lgs spel -- search` output for decoded entry names. That command prints
transaction diagnostics, not entries, so the grep always failed.

**Fix:** `3d8fd86` asserts `Transaction confirmed` at that point. The returned
entries are asserted downstream through the browser and the HTTP API, which read
the same wallet and sequencer state.

## Bug 2 — API read a stale program ID

**Failing:** run `44781784603` attempt 2. The browser found the entry, but the
API returned HTTP 500 on `/index`. The lifecycle deploys the freshly built
program (`E2E_PROGRAM_ID = 5f80583…`, derived from the clean build), while
`registry-api` inherited `REGISTRY_PROGRAM_ID = 4df7d539…` from CI env — a
different build path. The API tried to read an index account that does not exist
on the fresh sequencer. The failing artifact
([14656308333](https://github.com/goldennftplatform-svg/proclaim/actions/artifacts/14656308333))
shows the browser returning the entry while `api.log` prints the wrong ID.

**Fix:** `1015a8a` launches the API with `REGISTRY_PROGRAM_ID="$PID"`, pinning it
to the program this run actually deployed. Next run was green.

## Honest scope

- `RISC0_DEV_MODE=1` is set. This is a lifecycle/read test, **not** a real-proving
  result and not a proof-cost measurement.
- The sequencer is a fresh local test node. **Nothing was deployed to a public
  chain**, including the live Logos testnet.
- The registry tools remain local/review artifacts. No third party has adopted
  them, and no prize criterion is claimed by this run.
- The artifacts are debug identities and disposably funded accounts from the LEZ
  Justfile, not real funds.

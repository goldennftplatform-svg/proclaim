# Solution: LP-0023 — proclaim: LEZ Program Registry with reproducible source verification

**Submitted by:** <Your name or team name>

> **Status: early draft submission.** This PR is opened early to establish a submission timestamp and start community review while the testnet 0.4 deployment and the adoption push continue, as the prize encourages ("release early"). Several success criteria and **all** Adoption criteria are **not yet met**; they are marked unchecked below and listed under [Pending work](#pending-work). Nothing in this PR should be read as claiming a criterion that is not checked.

> **Spec check (2026-11-19):** the published LP-0023 spec on `master` still requires SPEL with a SPEL-generated IDL and already targets testnet 0.4. No competing LP-0023 solution PR exists. PRs #188 (would replace SPEL with a LEZ actor program and `lez-idl`) and #189 (release-integrity/signing policy) are open but unmerged, so neither is treated as current spec; both must be rechecked immediately before submission. Full findings: [`docs/SPEC-RECHECK.md`](https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/SPEC-RECHECK.md).

> **Testnet 0.4 status (2026-11-19):** Logos has **not published** a stable 0.4 release, sequencer endpoint, faucet, or explorer. The official endpoint table lists every 0.4 field as `TBD`, the newest LEZ release is `v0.4.0-rc3` (pre-release), and the official roadmap says 0.4 depends on LEZ programs landing **30 Nov 2026**. A 0.4 end-to-end deployment is therefore **externally blocked**, not omitted. Full evidence and the exact port/deploy runbook for when 0.4 is published: [`docs/TESTNET-0.4-READINESS.md`](https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/TESTNET-0.4-READINESS.md).

## Summary

`proclaim` is a LEZ Program Registry built with SPEL. It turns an opaque LEZ program ID into a discoverable entry with a name, version, author, description, tags, and resolvable pointers to an IDL and source on Logos Storage, and it records an independently reproducible source-verification attestation.

The deliverables are:

- an **on-chain registry program** (six instructions: `init_index`, `register`, `update`, `mark_verified`, `lookup`, `search`) with one PDA per program ID and a global index PDA;
- a Rust **SDK** (`registry_core`) with Borsh/serde types, PDA derivation helpers and search filters, plus a generated C **FFI** client and header;
- an IDL-driven **CLI** (`lgs spel` / `make cli`) covering register, update, lookup, search and source verification, plus a read-only `registry_browser` enumerator;
- a read-only Axum **HTTP API** (`registry-api`) on `127.0.0.1:3030` and a dependency-free **JS/TS client** (`clients/js`), with a 15-minute integration tutorial;
- a reusable **QtRO storage upload client** (`tools/qtro_storage_upload.py`) for publishing IDL/source to Logos Storage;
- a clean-room **reproducible-build verifier** (`scripts/verify-source.sh`), a **clean-environment demo script** (`scripts/demo-clean.sh`), documented design choices, and a Qt/QML Basecamp module that builds into a **cryptographically signed, verified `.lgx`** package (`docs/LGX-BUILD.md`).

The full lifecycle has been executed end-to-end against a local standalone sequencer: deploy → `init-index` → `register` → `search` → independent `mark-verified` → `inspect`. Authorization and integrity properties were exercised: self-verification is rejected, unauthorized update is rejected, and a source-changing update clears any prior attestation. The IDL and source archive were uploaded to Logos Storage and their real CIDs written on-chain. A clean rebuild reproduced the deployed program ID byte-for-byte.

## Repository

- **Repo:** <https://github.com/goldennftplatform-svg/proclaim>
- **Commit referenced by this submission:** latest `master` HEAD (this early draft is updated as work lands; the exact commit will be pinned when the PR is finalized).
- **License:** dual licensed under the MIT License and Apache License 2.0 (`LICENSE-MIT`, `LICENSE-APACHE`; crates and new source files declare `MIT AND Apache-2.0`).
- **IDL:** [`registry-idl.json`](https://github.com/goldennftplatform-svg/proclaim/blob/master/registry-idl.json), generated from the SPEL annotations via `make idl`. The frozen upload copy has SHA-256 `ef4fddc41d098d8360f4685df15f0e7dee33a0ff8dbce65041603dfac8cc4ee1` and matches the checked-in IDL byte-for-byte.
- **Status / evidence log:** [`LP0023.md`](https://github.com/goldennftplatform-svg/proclaim/blob/master/LP0023.md) records program IDs, PDAs, transaction hashes and CIDs.
- **CI:** [`.github/workflows/ci.yml`](https://github.com/goldennftplatform-svg/proclaim/blob/master/.github/workflows/ci.yml) — fmt, clippy, Rust unit tests, JS tests and a full real-sequencer lifecycle E2E job on every push/PR (green-run link is recorded under the E2E criterion once available).

## Approach

**Data model.** Each program gets its own PDA derived from the program ID (`#[account(init, pda = arg("key"))]`), so lookup by program ID is a direct derivation and needs no indexer. A single global `RegistryIndex` PDA (seed `registry_index`) holds the list of program IDs, which clients enumerate to implement `search`. The entry is a Borsh `RegistryEntry` holding metadata and pointers only: `program_id`, `name`, `version`, `author`, `description`, `tags`, `idl_cid`, `code_cid`, `commit`, `build_config`, `timestamp`, `deployer_claimed`, and the attestation fields `source_verified`, `verifier`, `verification_method`, `evidence_cid`. Source code and the IDL live off-chain on Logos Storage; on-chain storage is deliberately limited to metadata and pointers to keep gas cost low.

**Authorization.** `update` requires the original registering author as signer. `mark_verified` requires a verifier distinct from the author and records the method and an evidence CID. Any change to `code_cid`, `idl_cid`, `commit` or `build_config` clears the attestation, so a stale verification cannot survive a source change. Both rejection paths were exercised on-chain.

**Source verification.** `scripts/verify-source.sh` rebuilds the guest from a clean target directory, wraps the ELF in R0BF, derives the program ID and compares it with the recorded ID. For the submitted commit the rebuild matched byte-for-byte (SHA-256 of the frozen source archive `cf52946044cc04d4b04028f14ab6c0c4ab7b900f116696e807dc850be4a68b89`). The independent attestation was then written on-chain from a different account with method `reproducible-rebuild:program-id` and the source archive CID as evidence.

**What did not work / known gaps.** `make build` (cargo-risczero) requires Docker, which was unavailable; the guest was instead built with the local RISC Zero toolchain (`cargo +risc0`) and wrapped with the local `make-bin` R0BF helper, which is the documented reproducible path. Private transactions failed with `Failed to prove program: invalid receipt format` with consistent dependency pins; this remains unresolved upstream and does not affect the public registry operations. The storage module is a QtRO 2.0 source on a Unix socket rather than an HTTP service, and PySide6 6.12 does not assign a dynamic replica's real metaobject, so method return values are unavailable from Python; the upload client therefore recovers CIDs from the `storageUploadDone` event with a fallback that parses `manifestCid=` lines from the host log (`--host-log`). The isolated storage node could not reach the public bootstrap peers from this environment, so the uploaded bytes are currently cached locally rather than network-retrievable — see Pending work.

**Design choices left to the submitter.**

- *What lives on-chain:* metadata, pointers and the attestation record. This makes the entry self-describing and auditable while keeping writes small; the IDL and source are content-addressed on Logos Storage.
- *How verified source is hosted:* both — the source is on a public repository (so it can be inspected with ordinary tools) and mirrored to Logos Storage at CID `zDv*yXuozL` (so re-verification does not depend solely on a third-party repository's liveness). The on-chain entry records both the commit and the CID.
- *Trusted-signer overlay:* none is hardcoded. Any account distinct from the author may submit a verification, and the verifier, method and evidence CID are all recorded on-chain and user-inspectable. Clients may choose which attestations to weigh; the protocol does not impose a trust root.

**Why Logos.** A registry is shared substrate: every wallet, explorer and developer tool would otherwise ship its own drifting hardcoded list. Putting the index and attestation on LEZ makes entries trustless and publicly auditable, and using content-addressed Logos Storage means a source or IDL pointer cannot silently change content. A centralised alternative would reintroduce a single party able to alter metadata or withhold the index, and would lose the ability to bind a verification to the actual deployed program ID.

## Success Criteria Checklist

Legend: `[x]` = met with evidence in the repo / on chain; `[ ]` = not yet met (pending, explained under Pending work).

### Functionality

- [x] **On-chain registry built with SPEL, published IDL.** Six instructions in `methods/guest/src/bin/registry.rs`; IDL generated by `make idl` and tracked at `registry-idl.json`. On-chain entries hold metadata and pointers; source and IDL live on Logos Storage at CIDs `zDv*yXuozL` and `zDv*8iXZ6U`.
- [x] **Register.** Any account may register a program it deployed, supplying name, version, author, description, IDL pointer, code pointer and a timestamp. Lookup derives the entry PDA directly from the program ID, so it does not depend on a centralized indexer.
- [ ] **Deployer vs third-party entries.** The entry carries an on-chain `deployer_claimed` flag and any account may register; unauthorized **update** and self-**verification** attempts are rejected and demonstrated (`LP0023.md`). However, the deployer claim is currently self-asserted by the signer rather than cryptographically bound to the deployment, and there is no demo of an unauthorized deployer entry being rejected. Pending.
- [x] **Update.** Only the original registering author may update version, IDL pointer and description; an unauthorized update was rejected on-chain. Authorship transfer (optional) is not implemented.
- [x] **IDL publishing to Logos Storage.** Real CID `zDv*8iXZ6U` recorded on-chain (`update` tx `90e972de2f2b3dede86f52eead470d5fbe070078f90b0508ae33fe4a35896fbe`). Network retrieval is tracked separately below.
- [x] **Discovery: lookup / enumerate / search by name and author.** `lookup` derives the entry PDA; `search` reads the global index; `registry_browser` (`--name`), the HTTP API (`/search?name=…`) and the JS client (`search({ name })`) all support case-insensitive substring search by name, plus tag (OR), author and verification filters. Filter kinds compose with AND. Covered by SDK and JS unit tests.
- [x] **Source verification.** The on-chain entry records a source CID, commit and build configuration; `scripts/verify-source.sh` reproduces the program ID byte-for-byte from a clean build; an independent `mark_verified` from a different account recorded method `reproducible-rebuild:program-id` and evidence CID `zDv*yXuozL` (tx `4d31a6cf…`). A source-changing update clears the attestation (tested).
- [ ] **Testnet 0.4.** Everything above was executed against a local standalone sequencer (`127.0.0.1:3040`). No testnet 0.4 deployment exists. This is **externally blocked**: no stable 0.4 release, endpoint, faucet or explorer is published as of 2026-11-19, and the official roadmap targets 30 Nov 2026. Evidence and a ready port/deploy runbook: [`docs/TESTNET-0.4-READINESS.md`](https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/TESTNET-0.4-READINESS.md). Still unchecked.

### Usability

- [x] **Module / SDK for other modules and third-party apps.** `registry_core` (Rust SDK with types, PDA helpers and search filters), generated C FFI (`registry_ffi/generated/registry.h`, `registry_client.rs`), read-only HTTP API (`registry-api`) and a dependency-free JS/TS client (`clients/js`), with a worked example in [`docs/INTEGRATION.md`](https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/INTEGRATION.md).
- [x] **CLI covering register, update, lookup, search and source verification.** The IDL-driven `lgs spel` / `make cli` interface exposes all six instructions; `make browser` provides a read-only enumerator with name/tag/author/verified filters.
- [x] **Logos Core Module published to a catalog.** The Qt/QML module in `ui/registry/` builds into a **valid signed `.lgx`** (`lgx verify`: structure + signature valid; [`docs/LGX-BUILD.md`](https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/LGX-BUILD.md)), published as a [GitHub Release asset](https://github.com/goldennftplatform-svg/proclaim/releases/tag/registry-v0.1.0) and listed in the [`proclaim-modules` catalog](https://github.com/goldennftplatform-svg/proclaim-modules) via [`logos-repo.json`](https://raw.githubusercontent.com/goldennftplatform-svg/proclaim-modules/main/logos-repo.json) + generated [`index.json`](https://raw.githubusercontent.com/goldennftplatform-svg/proclaim-modules/main/index.json). The catalog passes the official `index.py validate-repo` and `validate --full` checks (index↔file `rootHash`, manifest and signer DID bindings). **Remaining caveats:** packaged with the standalone `lgx` CLI, not `logos-module-builder` (separate criterion); the signer is a self-generated dev key trusted by this catalog only; not yet install-tested in Basecamp.

### Reliability

- [x] **Graceful degradation for unresolvable / unparseable IDLs.** Registry metadata lives on-chain and is rendered without fetching the IDL at all, so a missing or malformed IDL body cannot break entry display. Both SDKs make this explicit: `registry_core::RegistryEntry::idl_availability()` and the JS `idlAvailability()` report empty/placeholder CIDs as `Unavailable`/`'unavailable'`, with unit tests in `registry_core` (7 passing) and `clients/js` (8 passing).
- [x] **No mandatory centralized external services.** Beyond the configured sequencer and Logos Storage, no third-party web service is required; the repository contains no analytics or telemetry.

### Performance

- [x] **Cycle counts for register and update.** Measured against the exact deployed R0BF artifact (program ID `4df7d539…65144`, ELF SHA-256 `88716a6b…2506c5`) with a proclaim-specific executor harness that asserts both before measuring. **`register`: 319,445 user cycles; `update`: 226,298** (also `init_index` 86,937, `mark_verified` 230,009, `lookup` 169,398, `search` 164,656; 20 samples, 1 segment each). LEZ `cycle_bench` cannot load an arbitrary program ID, hence the custom harness — full commands, fixtures and provenance in [`docs/CYCLES-HARNESS-RESULT.md`](https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/CYCLES-HARNESS-RESULT.md) and the table in [`docs/PERFORMANCE.md`](https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/PERFORMANCE.md). Caveats: synthetic fixed-size fixtures; po2/proving totals not measured (reported `null`, not fabricated).

### Supportability

- [ ] **Deployed and tested against testnet 0.4.** Pending.
- [ ] **Platform support.** Linux verified. macOS, Windows and Android not yet covered (macOS is checked through Adoption). Pending.
- [ ] **Logos Core Module built with `logos-module-builder` (`mkLogosModule`).** Still pending: the module is packaged with the standalone `lgx` CLI, not the Nix builder. The builder's QML helper is actually named `mkLogosQmlModule` (`ui_qml`), while the prize text says `mkLogosModule`; reviewer interpretation is unresolved — see [`docs/MODULE-PACKAGING-RESEARCH.md`](https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/MODULE-PACKAGING-RESEARCH.md) and [`docs/LGX-BUILD.md`](https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/LGX-BUILD.md).
- [x] **IDL generated via SPEL; registry registered in itself.** `registry-idl.json` is generated from `#[lez_program]` annotations, and the registry's own program ID is registered as an entry ("LP-0023 Registry v5").
- [ ] **E2E integration tests against a real sequencer in CI, green on default branch.** A CI workflow now runs fmt, clippy, Rust unit tests and JS tests on every push/PR ([`docs/CI.md`](https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/CI.md)), but the real-sequencer E2E job is manual and only a read smoke test because no always-on sequencer or dedicated wallet is available. Still pending.
- [x] **README documents setup, program ID and step-by-step CLI / module usage.** [`README.md`](https://github.com/goldennftplatform-svg/proclaim/blob/master/README.md) and [`LP0023.md`](https://github.com/goldennftplatform-svg/proclaim/blob/master/LP0023.md) do so for the localnet deployment; the testnet 0.4 program ID is pending.
- [x] **SDK worked example resolving a program ID to metadata and IDL.** [`docs/INTEGRATION.md`](https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/INTEGRATION.md) and the JS client README.

### Adoption (required) — all pending

- [ ] **5 independent third-party Logos UI modules** integrating the SDK: none yet (0/5).
- [ ] **50 registered programs from at least 20 distinct independent author accounts** on the official zone: currently **1** entry from **1** author, on localnet (1/50, 1/20).
- [ ] **20 entries with a verified source claim:** currently **1** (1/20).
- [ ] **Sustained activity:** ≥10 registrations in each of ≥2 months: not started (0 months).
- [ ] **20 Discord testimonials:** none yet (0/20).
- [ ] **20 Twitter/X testimonials:** none yet (0/20).
- [ ] **macOS coverage** in adoption evidence: none yet.

## FURPS Self-Assessment

### Functionality

Six on-chain instructions cover the registry lifecycle: initialize the global index, register an entry, author-only update, independent verification, and two read paths. Clients derive an entry PDA from a program ID and enumerate the global index for search. SDK, CLI, HTTP API, JS client, FFI header and a reproducible-build verifier are provided. Name and author search, and graceful handling of unusable IDL pointers, are covered by unit tests. Cycle counts are measured (`register` 319,445, `update` 226,298 user cycles; [result](https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/CYCLES-HARNESS-RESULT.md)). Limitations: no authorship transfer, no cryptographic deployer-claim binding yet, no testnet-0.4 deployment, no po2/proving totals, and no automatic real-sequencer E2E in CI yet.

### Usability

The CLI is IDL-generated from the program source, so its arguments cannot drift from the on-chain interface. `registry_browser` and the HTTP API provide read-only discovery with tag/author/verified filters. `docs/INTEGRATION.md` is a copy-paste 15-minute guide covering PDA derivation, record fields, storage upload, registration, search and verification rules. The Qt/QML module compiles into a signed `registry.lgx` ([build record](https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/LGX-BUILD.md)), published as a [Release asset](https://github.com/goldennftplatform-svg/proclaim/releases/tag/registry-v0.1.0) and listed in the [proclaim catalog](https://github.com/goldennftplatform-svg/proclaim-modules); a Basecamp install test is still pending.

### Reliability

Authorization is enforced on-chain: only the original author can update; verification requires a signer distinct from the author; and a source-changing update clears any prior attestation. Reads degrade independently of CID resolution because metadata is stored on-chain, and unusable IDL pointers are handled explicitly in both SDKs with unit tests. No analytics or third-party web dependencies.

### Performance

On-chain writes are bounded to metadata and pointers. The account-access model and the measured cycle counts (`register` 319,445, `update` 226,298 user cycles) are in [`docs/PERFORMANCE.md`](https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/PERFORMANCE.md); LEZ `cycle_bench` cannot load an arbitrary program ID, so a [proclaim-specific harness](https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/CYCLES-HARNESS-RESULT.md) executes the exact deployed artifact. Off-chain search reads a single index account and derives/fetches entry accounts, so query cost scales linearly with the number of entries and requires no indexer service.

### Supportability

The workspace is split into shared types, guest, SDK, FFI, examples, HTTP API, JS client and UI scaffold. Build, IDL generation, FFI generation and UI generation are documented in the README, with a Docker-free reproducible guest build path. CI (`.github/workflows/ci.yml`) runs fmt, clippy, Rust and JS unit tests on every push/PR, and `scripts/demo-clean.sh` provides a deterministic offline verification plus read-only online checks; `verify-source.sh` provides an independently runnable clean-room rebuild check. The storage upload client is reusable and its authentication protocol and PySide6 limitations are documented.

## Supporting Materials

- [README — setup, make targets, project structure](https://github.com/goldennftplatform-svg/proclaim/blob/master/README.md)
- [LP0023.md — IDs, PDAs, transaction hashes, CIDs, tested commands](https://github.com/goldennftplatform-svg/proclaim/blob/master/LP0023.md)
- [IDL](https://github.com/goldennftplatform-svg/proclaim/blob/master/registry-idl.json)
- [Integration tutorial](https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/INTEGRATION.md)
- [Roadmap — Phase A–E](https://github.com/goldennftplatform-svg/proclaim/blob/master/ROADMAP.md)
- [Performance and cost notes](https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/PERFORMANCE.md)
- [CI workflow and honest E2E status](https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/CI.md)
- [Clean-environment demo script](https://github.com/goldennftplatform-svg/proclaim/blob/master/scripts/demo-clean.sh) — regenerates the IDL, runs both test suites and optionally performs read-only sequencer/API checks.
- [LP-0023 spec / PR status re-check](https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/SPEC-RECHECK.md)
- [Module packaging / catalog research](https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/MODULE-PACKAGING-RESEARCH.md)
- [Signed `.lgx` build + verification record](https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/LGX-BUILD.md)
- [Published `registry 0.1.0` Release asset](https://github.com/goldennftplatform-svg/proclaim/releases/tag/registry-v0.1.0)
- [proclaim module catalog (`logos-repo.json`)](https://github.com/goldennftplatform-svg/proclaim-modules)
- [Cycle harness result — measured register/update cycles](https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/CYCLES-HARNESS-RESULT.md)
- [Cycle counts + automatic E2E CI plan](https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/CYCLES-E2E-PLAN.md)
- [Automatic E2E implementation plan](https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/E2E-CI-PLAN.md)
- [Testnet 0.4 readiness and port/deploy runbook](https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/TESTNET-0.4-READINESS.md)
- [Adoption tracker with honest current counts](https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/adoption/ADOPTION_TRACKER.md)
- [Third-party module starter](https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/adoption/THIRD_PARTY_MODULE_STARTER.md) and [builder-hub post draft](https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/adoption/BUILDER_HUB_POST.md)
- [Clean-room source verifier](https://github.com/goldennftplatform-svg/proclaim/blob/master/scripts/verify-source.sh)
- [QtRO storage upload client](https://github.com/goldennftplatform-svg/proclaim/blob/master/tools/qtro_storage_upload.py) and [isolated host launcher](https://github.com/goldennftplatform-svg/proclaim/blob/master/tools/launch_storage_host.sh)
- [HTTP API](https://github.com/goldennftplatform-svg/proclaim/tree/master/registry-api) and [JS client](https://github.com/goldennftplatform-svg/proclaim/tree/master/clients/js)

On-chain records (localnet standalone sequencer):

| Item | Value |
| --- | --- |
| Program ID (v5) | `4df7d5396b68a16aae6d0f768ef447a0055eb43866827154f71139e486265144` |
| Index PDA | `9mr5hePKo24ZHNztzVpuRND7PeAsR17yQ2M5HHPKcbw3` |
| Entry PDA | `EP644ahCbBehPMV8krDKDnToxJwfaMGaKKa1NzDFcM6q` |
| Deploy tx | `b8c121462dedb211e55551523d68b3568bc62c88b4518676bc4fd2e2acb03e51` |
| Register tx | `38bba96b9bd663d83a7ec6dbefc20b6ddf800be908dda0c6590b898b23b9d135` |
| Update tx (real CIDs) | `90e972de2f2b3dede86f52eead470d5fbe070078f90b0508ae33fe4a35896fbe` |
| Verify tx (final) | `4d31a6cfe23d9e82425ea6cde250c2b17384c0fcbbb14ecaf185508bfb5cd88f` |
| IDL CID | `zDv*8iXZ6U` |
| Source archive CID | `zDv*yXuozL` |

## Pending work

1. Bind a deployer claim to the deployment signer and add a demo showing an unauthorized deployer entry being rejected.
2. Make uploaded CIDs network-retrievable (run a publicly reachable storage node seeded with the IDL and source), then prove retrieval by an independent node.
3. Deploy to Logos testnet 0.4 once available; record the program ID and rerun the lifecycle and verification there.
4. Install-test the published module in Basecamp from the [`proclaim-modules` catalog URL](https://raw.githubusercontent.com/goldennftplatform-svg/proclaim-modules/main/logos-repo.json). Also decide whether to rebuild via `logos-module-builder`: its QML path uses `mkLogosQmlModule` (`ui_qml`), while the prize text names `mkLogosModule` — an unresolved reviewer-interpretation question documented in `docs/MODULE-PACKAGING-RESEARCH.md`. The `.lgx` is already published as a [Release asset](https://github.com/goldennftplatform-svg/proclaim/releases/tag/registry-v0.1.0) and in the catalog; the dev signing key is trusted by that catalog only.
5. Make the real-sequencer E2E job automatic and green on the default branch (implementation files staged and locally ground-checked in `staged/e2e-ci/`, plan in `docs/E2E-CI-PLAN.md`; host-side CI is already in place); extend platform coverage.
6. Adoption: recruit 5 independent third-party UI modules, drive 50 registrations from 20 independent authors over ≥2 months (≥10/month), reach 20 verified source claims, collect 20 Discord and 20 X testimonials, and include macOS users/installs.

## Terms & Conditions

By submitting this solution, I confirm that I have read and agree to the [Terms & Conditions](../TERMS.md).

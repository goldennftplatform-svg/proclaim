# LP-0023 — Roadmap beyond localnet

Status snapshot: 2026-11-19. Registry published at
https://github.com/goldennftplatform-svg/proclaim (MIT OR Apache-2.0).

## Phase A — Close the local deliverable (complete)
- [x] On-chain registry: 6 instructions, IDL, FFI, generated UI scaffold.
- [x] `registry_browser` read-only enumerator (name/tag/author/verified filters).
- [x] Name search end-to-end: `registry_core::search_entries_with_name`,
      `registry_browser --name`, `/search?name=…`, `client.search({ name })`.
- [x] Graceful IDL degradation: `idl_availability()` / `idlAvailability()` with
      unit tests (7 Rust + 8 JS passing).
- [x] `docs/PERFORMANCE.md` cost model with **measured** cycles: `register`
      319,445 / `update` 226,298 user cycles (plus 4 other instructions), from
      the proclaim harness in `docs/CYCLES-HARNESS-RESULT.md`. LEZ `cycle_bench`
      cannot load our program ID; po2/proving totals not measured.
- [x] Clean-room `verify-source.sh` reproduces the v5 program ID.
- [x] Repo published with dual licenses, no secrets, reproducible-build docs.
- [x] **QtRO storage client** → uploaded IDL + source, obtained real CIDs
      (`zDv*8iXZ6U`, `zDv*yXuozL`) with `tools/qtro_storage_upload.py`.
- [x] Update on-chain entry with real `idl_cid` / `code_cid` / evidence CID
      (`update` is author-only; a source-field change clears any prior attestation).
- [x] Re-ran `verify-source.sh` (byte-for-byte match), submitted independent
      `mark-verified` from the verifier account. `registry_browser --verified`
      shows the entry.
- [x] Commit the reusable QtRO upload helper under `tools/` (+ host launcher).

## Phase B — Testnet 0.4 deployment
- [ ] Obtain testnet sequencer endpoint + funded deployer wallet.
- [ ] Rebuild guest from published source, reproduce program ID, deploy.
- [ ] `init-index`, `register` the registry itself, register ≥1 third-party sample.
- [ ] Publish the program ID + deployed account addresses in `LP0023.md`.
- [ ] Repeat `verify-source.sh` + independent `mark-verified` on testnet.

## Phase C — Third-party UI modules (prize requires 5)
- [x] Build a **signed, verified** `registry.lgx` from `ui/registry/` with the
      local `lgx` CLI (no Nix/Docker); fixes included the C++ `register`
      keyword and preserving generated manifest hashes. Full record:
      `docs/LGX-BUILD.md`.
- [ ] Publish the `.lgx` as a GitHub Release asset + `logos-modules-release-base`
      catalog (`logos-repo.json`), then install-test in Basecamp. Decide whether
      to rebuild via `logos-module-builder` (`mkLogosModule`/`mkLogosQmlModule`).
- [ ] Recruit 5 independent UI/module authors to consume the registry client
      (FFI header + `registry_core` SDK are the integration surface).
- [ ] Track each module's repo + the registry entries it registers.

## Phase D — Adoption metrics (prize requires sustained use)
- [ ] 50 registered programs from 20 distinct authors.
- [ ] 20 byte-verified source claims (independent `mark-verified` attestations).
- [ ] ≥10 registrations/month for two consecutive months.
- [ ] 20 Discord + 20 X/Twitter testimonials with links archived.
- [ ] macOS coverage: verify `verify-source.sh` and the FFI build on macOS.

## Phase E — Submission readiness
- [x] Draft the solution PR body (`docs/SOLUTION-LP-0023.md`) and submission
      steps (`docs/SUBMIT-LP-0023.md`), modelled on solution PR #187's
      early-draft pattern. Not yet opened in `logos-co/lambda-prize`.
- [x] CI workflow with fmt/clippy/unit tests on push/PR (`.github/workflows/ci.yml`);
      manual real-sequencer E2E job documented in `docs/CI.md`.
- [ ] Automatic real-sequencer E2E on every push: implementation files are
      staged and locally ground-checked (`staged/e2e-ci/`, plan in
      `docs/E2E-CI-PLAN.md`), but NOT yet applied to the repo and NOT yet proven
      by a green default-branch GHA run. Do not mark this complete until a
      successful run is linked.
- [x] Clean-environment demo script (`scripts/demo-clean.sh`) with offline and
      read-only online modes.
- [x] Adoption outreach kit in `docs/adoption/` (builder-hub post, third-party
      module starter, honest adoption tracker).
- [x] Re-check the published LP-0023 spec and open PRs (`docs/SPEC-RECHECK.md`):
      no competing solution PR; #188/#189 spec/policy changes still unmerged.
- [x] Verify testnet 0.4 readiness (`docs/TESTNET-0.4-READINESS.md`): 0.4 is
      externally blocked (all endpoints `TBD`, only RCs, roadmap targets
      30 Nov 2026). Port/deploy runbook recorded for when 0.4 is published.
- [ ] Ensure every criterion in the LP-0023 spec maps to a linked artifact.
- [ ] Public posting history / account history checks: keep all work in the
      published repo and public testnet, no manufactured volume.
- [ ] Submit PR against the λPrize registry per LP-0023 rules (first complete
      solution meeting every criterion wins; paid in USDT).

## Notes on dependencies
- Phases C–D require real external users and elapsed time; they cannot be
  completed from this machine alone. Phase A–B are fully local/controllable and
  are the current focus.
- Do not restart Basecamp again unless required: 0.3.2 full-validation replay is
  expensive. The blockchain module must be re-opened with
  `LogosBasecamp --uri 'basecamp://open/blockchain_ui'` after any restart, and
  mining re-enabled with `PUT /pow/mining/start`.

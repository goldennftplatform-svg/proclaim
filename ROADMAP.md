# LP-0023 — Roadmap beyond localnet

Status snapshot: 2026-11-19. Registry published at
https://github.com/goldennftplatform-svg/proclaim (MIT OR Apache-2.0).

## Phase A — Close the local deliverable (in progress)
- [x] On-chain registry: 6 instructions, IDL, FFI, generated UI scaffold.
- [x] `registry_browser` read-only enumerator (tag/author/verified filters).
- [x] Clean-room `verify-source.sh` reproduces the v5 program ID.
- [x] Repo published with dual licenses, no secrets, reproducible-build docs.
- [ ] **QtRO storage client** → upload IDL + source, obtain real CIDs.
      - Toolchain: PySide6 `QtRemoteObjects` (installing), or a small C++ client
        linked against the AppImage's `libQt6RemoteObjects.so.6`.
      - Socket: `/tmp/logos_storage_module_<hash>` (QtRO 2.0 / QDataStream).
      - Methods: `uploadUrl(path, chunkSize, advertise)`, then read the CID from
        the `storageUploadDone` event; confirm with `exists(cid)` / `manifests()`.
- [ ] Update on-chain entry with real `idl_cid` / `code_cid` / evidence CID
      (`update` is author-only; a source-field change clears any prior attestation).
- [ ] Re-run `verify-source.sh`, then submit independent `mark-verified` from the
      verifier account. Confirm `registry_browser --verified` shows the entry.
- [ ] Commit the reusable QtRO upload helper into the repo under `scripts/`.

## Phase B — Testnet 0.4 deployment
- [ ] Obtain testnet sequencer endpoint + funded deployer wallet.
- [ ] Rebuild guest from published source, reproduce program ID, deploy.
- [ ] `init-index`, `register` the registry itself, register ≥1 third-party sample.
- [ ] Publish the program ID + deployed account addresses in `LP0023.md`.
- [ ] Repeat `verify-source.sh` + independent `mark-verified` on testnet.

## Phase C — Third-party UI modules (prize requires 5)
- [ ] Package the generated `ui/registry/` module as an installable `.lgx` and
      document the install path for Basecamp users.
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

# LP-0023 (LEZ Program Registry) — spec/status re-check

**Date checked:** 2026-11-19
**Method:** public, unauthenticated GitHub REST API + raw GitHub content fetches. No token was used.
**Sources checked:**
- Open PRs: https://api.github.com/repos/logos-co/lambda-prize/pulls?state=open&per_page=100
- Current spec: https://raw.githubusercontent.com/logos-co/lambda-prize/master/prizes/LP-0023.md
- Spec history: https://api.github.com/repos/logos-co/lambda-prize/commits?path=prizes/LP-0023.md&per_page=10
- PRs #188 / #189: API PR objects, `.diff`, issue comments and review comments
- Title search for LP-0023 PRs (incl. closed/merged): https://api.github.com/search/issues?q=repo%3Alogos-co%2Flambda-prize+is%3Apr+LP-0023+in%3Atitle&per_page=100

**Rate limit:** none of the API calls returned a rate-limit error; all requested data was returned. These were unauthenticated requests, so no elevated quota was used.

---

## 1. Open PRs in `logos-co/lambda-prize`

The API returned **15 open PRs** (sorted by API default, newest first). Full list:

| PR | Title | Draft | Author | Notes |
|----|-------|-------|--------|-------|
| [#189](https://github.com/logos-co/lambda-prize/pull/189) | Release integrity and security requirements for submissions | Yes | `weboko` | Policy PR; **open, not merged**. See §5. |
| [#188](https://github.com/logos-co/lambda-prize/pull/188) | Update for lez 0.4, and updated idl | No | `jzaki` | LP-0023 spec-change PR; **open, not merged**. See §4. |
| [#187](https://github.com/logos-co/lambda-prize/pull/187) | Solution: LP-0018 — logos-osm: OpenStreetMap registry, SDK, CLI and Basecamp app on LEZ | Yes | `YodHeVauHe` | Solution for **LP-0018**, not LP-0023. |
| [#186](https://github.com/logos-co/lambda-prize/pull/186) | Solution: LP-0026 — Agora native forum for Logos Basecamp | No | — | Other prize. |
| [#184](https://github.com/logos-co/lambda-prize/pull/184) | LP-0029: Lending & Borrowing App | No | — | Other prize. |
| [#183](https://github.com/logos-co/lambda-prize/pull/183) | LP-0029: Private Messaging App | Yes | — | Other prize. |
| [#181](https://github.com/logos-co/lambda-prize/pull/181) | LP-0021/LP-0022: replace faucet mini-app with open funding path and EmPoWer | No | — | Other prizes. |
| [#179](https://github.com/logos-co/lambda-prize/pull/179) | Solution: LP-0026 - Forumer: a private, serverless forum for Logos Basecamp | No | — | Other prize. |
| [#168](https://github.com/logos-co/lambda-prize/pull/168) | Solution: LP-0026 - Logos Forum | No | — | Other prize. |
| [#166](https://github.com/logos-co/lambda-prize/pull/166) | LP-0028: Sustained Atomic-Swap Maker Operation on the Logos testnet | Yes | — | Other prize. |
| [#165](https://github.com/logos-co/lambda-prize/pull/165) | LP-0027: Atomic-Swap Maker Operator Kit — three independent testnet makers | Yes | — | Other prize. |
| [#137](https://github.com/logos-co/lambda-prize/pull/137) | Define Adoption criteria | No | — | Repo/process PR. |
| [#135](https://github.com/logos-co/lambda-prize/pull/135) | LP-0024: Basecamp Theming — theme engine, format, and community themes | Yes | — | Other prize. |
| [#90](https://github.com/logos-co/lambda-prize/pull/90) | Add LP-0018 and LP-0019 atomic-swap prizes | Yes | — | Other prizes. |
| [#75](https://github.com/logos-co/lambda-prize/pull/75) | ΛPrize 0018 | Yes | — | Other prize. |

**Status of the two tracked PRs:** both are **open**; `merged_at` is `null` for both, `closed_at` is `null` for both. #189 is a **draft**, #188 is not a draft. Neither has labels or a milestone. Neither has been merged.

## 2. Competing LP-0023 solution PR

**No competing solution PR for LP-0023 exists as of this check.**
- No open PR titled `Solution: LP-0023 ...` (or otherwise identifying itself as an LP-0023 solution) in the 15 open PRs.
- The title search for PRs containing `LP-0023` returned 4 PRs, all closed/merged and all **spec/admin**, not solutions: [#185](https://github.com/logos-co/lambda-prize/pull/185) (merged 2026-10-20, platform wording), [#157](https://github.com/logos-co/lambda-prize/pull/157) (merged 2026-09-22, open the prize), [#155](https://github.com/logos-co/lambda-prize/pull/155) (merged 2026-09-17, disclaimer fix), [#127](https://github.com/logos-co/lambda-prize/pull/127) (merged 2026-09-17, original draft).
- Note: PR #188 changes the LP-0023 spec but is not a solution; PR #187 is an LP-0018 solution.

## 3. Current LP-0023 spec on `master`

Fetched successfully from the raw URL in §Sources. Header: **"LP-0023: LEZ Program Registry [OPEN]"**, `Status: Open`, `Logos Circle: N/A`, total prize **$2000**, effort **Medium**.

Master spec history (path-filtered commits API) shows the most recent change to `prizes/LP-0023.md` was commit [`dbf3365`](https://github.com/logos-co/lambda-prize/commit/dbf3365952afaae9ec256f803fa2c5116ac6b82c) on **2026-10-20** ("prizes: adopt recommended-platforms wording..."). The testnet retarget to 0.4 was commit [`ad608bf`](https://github.com/logos-co/lambda-prize/commit/ad608bf93f22fea4dc619f0b27bea99f98e97d58) on **2026-10-12** ("Target testnet 0.4 in all open prizes"). No spec commits after 2026-10-20 appear in that history.

### Success criteria as currently published (master)

**Functionality**
- **On-chain registry:** metadata stored on-chain as a LEZ program built with **SPEL**, with a published IDL; source and the entry's own IDL off-chain on **Logos Storage**; on-chain storage limited to metadata and pointers, gas-conscious.
- **Register:** any account may register a program *it deployed* with at minimum **name, version, author, description, IDL pointer, code pointer, registration timestamp**; lookup must not depend on a centralized indexer.
- **Deployer vs third-party:** both allowed; deployer entry must be signed by the deploying account and distinguishable client-side without an off-chain service; the demo must show an **unauthorised deployer-entry attempt rejected**; source verification should confirm the claim.
- **Update:** only the original registering author may update version/IDL pointer/description; authorship transfer optional but if implemented needs both signatures.
- **IDL publishing:** IDL stored on Logos Storage.
- **Discovery:** lookup by program ID, enumerate all entries, search by **name and author**, using only submitted software.
- **Source verification:** record source pointer, commit and reproducible build configuration; user can rebuild locally and mark "locally verified" for themselves; alternative bytecode-verification strategies may be accepted.
- **Testnet 0.4:** end-to-end on the Logos LEZ zone on testnet 0.4.

**Usability:** module/SDK for other Logos modules and third-party apps (register, resolve program ID → metadata/IDL, searches); CLI covering register, update, lookup, search, source verification.

**Reliability:** graceful IDL degradation (unresolvable or unparseable IDL → metadata still shown, IDL marked unavailable); **no mandatory centralised external services** beyond configured sequencer/LEZ node/Logos Storage — any third-party web service must be disclosed, disableable, and analytics strictly opt-in; other data-protection obligations may apply.

**Performance:** document **cycle counts** for on-chain register and update, using LEZ `cycle_bench` terminology.

**Supportability:** deployed/tested on testnet 0.4; platform support recommended (Linux + macOS; Windows + Android from 0.4) — **not a separate build gate**, macOS is checked via Adoption; Logos Core Module built with `logos-module-builder` (`mkLogosModule`); **IDL for the registry generated via SPEL, registry registered in itself**; e2e tests against a real standalone sequencer in CI, green on default branch; README with setup, deployed program ID, step-by-step CLI/Core-Module usage; SDK docs with a worked "resolve program ID → metadata and IDL" example.

**Adoption (all required in addition to FURPS):** 5 independent third-party Logos UI modules integrating the SDK, publicly forged with genuine spread-out commit history and independent of submitter; 50 registered programs on the official zone from ≥20 distinct independent author accounts, genuinely deployed with resolvable/parseable IDLs, no placeholders or duplicates, prior unrelated testnet activity lends weight; 20 entries with independently rebuildable verified source claims; registrations spread over ≥2 months with ≥10 new registrations in each month (verified from on-chain timestamps); 20 Discord + 20 Twitter/X testimonials with account-history checks; macOS coverage present in adoption evidence.

**Additional Design Choices left to submitter (must pick a position, document and justify):** what lives on-chain (source/IDL must be off-chain on Logos Storage); how verified source is hosted (third-party repo, Logos Storage mirror, or both, with reasoning); whether to ship a trusted-signer overlay (must be user-inspectable and overridable if shipped).

**Scope:** in scope includes the on-chain registry built with SPEL, IDL publishing to/resolution from Logos Storage, reproducible builds and verification tooling, deployer-vs-third-party claim authorisation, lookup/search (self-hostable indexer allowed), SDK/CLI/Core Module. Out of scope: curation/moderation/trust scores, running a public indexer as a service, payment/licensing/monetisation.

**Submission/evaluation:** dual MIT **and** Apache-2.0 licensing; registry's own IDL and testnet-0.4 deployed program ID; Logos Core modules published to a module catalog forked from `logos-modules-release-base` via `logos-modules-release-action`, including the catalog's `logos-repo.json` URL; evidence per adoption criterion incl. per-month counts; FURPS self-assessment. First-come-first-served, adoption window must elapse, clean-environment demo script must succeed, evaluators rebuild a sample of verified-source entries. Max 3 submissions per prize, at most one review/week; feedback is pass/fail.

### Comparison with the expectations carried into this check

Every criterion listed in the earlier expectation (SPEL registry + published IDL; register/deployer-vs-third-party/update/lookup/enumerate/search/IDL on Storage/source verification; SDK + CLI; module catalog; graceful IDL degradation; cycle counts; testnet 0.4, platforms, `logos-module-builder`, IDL via SPEL, self-registration, e2e CI, README, SDK worked example; all adoption thresholds incl. macOS) is **present verbatim in substance on master**. Two items in the current spec were **not in that expectation list** and should be checked against the draft:
1. **Reliability: "No mandatory centralised external services"** with disclosure/disable/opt-in-analytics requirements.
2. **"Additional Design Choices Left to the Submitter"** — three positions that *must* be picked, documented and justified, since reasoning is evaluated.

No other surprise changes were found on master. The testnet-0.4 requirement is already on master (since 2026-10-12); it is not something introduced by PR #188.

## 4. PR #188 — proposed (NOT merged) LP-0023 changes

PR #188 is open, `merged_at: null`, last updated 2026-11-05. Its diff touches only `prizes/LP-0023.md`. If merged, it would change the following:

1. **On-chain registry:** replace "LEZ program built with SPEL" with "**LEZ actor program (an `actor_entrypoint!` handler taking Borsh-encoded messages)**".
2. **New functionality criterion "IDL format and tooling":** IDLs use the [LEZ IDL format](https://github.com/jzaki/lez-prog-call/blob/main/idl.md) (`lez-idl`), generated from source (not hand-written), byte-for-byte regenerable from the recorded source and commit. Tooling (submission's own or built on existing) must:
   - generate an IDL from source: messages in Borsh order with field names/types/doc comments, actor-state layouts, events;
   - encode any message from typed input; decode messages, actor state and events into named fields;
   - **plan and simulate a call against current chain state** — required actors, signers, apply/revert, cycle count and fee;
   - send a call to any program with declared actors and a fee payer;
   - compute a program's `image_id` from a local build exactly as the program loader does and compare with the deployed program header's `image_id`.
3. **Usability/SDK:** SDK must also decode a program's messages, state and events using its IDL.
4. **Supportability:** registry's own IDL "generated from its source in the LEZ IDL format" instead of "generated via SPEL".
5. **Adoption:** 50-program entries must have resolvable IDLs that **parse as LEZ IDL** (rather than merely "resolvable, parseable").
6. **In Scope:** registry as LEZ actor program; add "any IDL, call and verification tooling the registry needs that existing tooling doesn't already provide".
7. **Resources:** replace SPEL entry with the LEZ IDL format link and `program_loader` link; future-testnet note refers to "LEZ IDL format" instead of "SPEL IDL format".

**Review state:** issue comments on #188 contain only an automated validation bot comment ("Validation passed", no prize-related changes). Review comments show unresolved discussion:
- maintainer `fryorcraken` (2026-11-04): "I assume this should point to spel once ready?"
- author `jzaki` (2026-11-05): says SPEL needed a rewrite, so mapping his LEZ v0.4 tooling back into SPEL is less clear; eventual rename to `lez-call idl` once logos-co owns `lez-prog-call`.
- reviewer `vpavlin` (2026-11-05): "That link throws 404 for me" (about the LEZ IDL format link).
No approval or resolution is visible; **do not treat #188's text as current spec**.

## 5. PR #189 — proposed (NOT merged, draft) release-integrity policy

PR #189 is open, **draft**, `merged_at: null`, created 2026-11-13. Its PR body explicitly states: **"Existing prize files (LP-0001, 0018, 0021, 0022, 0023, 0026) are not touched."** It modifies `README.md`, template `prizes/LP-0000.md`, template `solutions/LP-0000.md`, the validation script/workflow and `AGENTS.md`. The proposed README policy says it applies to "every prize whose deliverables include a Basecamp module, unless the prize states otherwise", and the PR body lists **scope in time for already-open prizes as an unresolved decision** ("Should it also apply to prizes that are open now, or only to prizes published or revised after merge (possibly with a grace date)?"). LP-0023 requires a Logos Core Module published via a module catalog, so this could affect the draft if merged with broad scope — but it is **not current master policy**.

Exact proposed requirements relevant to a solution draft:
- **Signed packages:** every `.lgx` release in the declared catalog signed with `lgx sign` (`signing_mode: inline` or `external`); signer DID listed under `trustedSigners` in `logos-repo.json`; unsigned releases not evaluated.
- **Signer declared up front:** `did:jwk:` DID in the solution PR and project README; other keys don't count; rotation must be announced before publishing.
- **Immutable GitHub releases:** enable release immutability on catalog (and source) repo before publishing; every submitted release must show the **Immutable** badge; rolling `index` release must already exist when immutability is enabled.
- **Freeze rule:** award is for the evaluated `rootHash`; never replace/delete/re-tag releases or assets; changes ship as new versions; replacing the evaluated build disqualifies.
- **Release Integrity manifest** in the solution PR: source commit SHA (40 hex), catalog `logos-repo.json` URL, package name + version, release URL, `rootHash` (64 hex), SHA-256 of `.lgx` (64 hex), built platform variants, signer DID, optional Logos Storage CID. Values taken from release sidecar JSON or `lgx manifest`; evaluators recompute.
- **Canonical install instructions:** exact catalog URL + signer DID + `rootHash`; never "search for" the module, add a catalog from a chat message, or disable signature warnings.
- **Catalog on a GitHub organisation account** (not personal); no rename/delete during evaluation/post-award; optional Storage CID in manifest.
- **Account/repo hygiene:** 2FA on all writers, protected default branch and release tags, **all third-party Actions and reusable workflows pinned to full commit SHAs** (incl. replacing `@v1`/`@v4` from `logos-modules-release-base`), signing key only in Actions secret/HSM/hardware token, never in repo/logs/PR-triggered workflows.
- **Secure dev basics:** no plaintext keys/seed phrases/tokens; committed lockfiles (`Cargo.lock`, `flake.lock`); verify dependencies are intended packages.
- **Safe-to-evaluate deliverables:** no `--accept-flake-config`, no extra Nix substituters/trusted keys, no unpinned remote scripts into a shell, no prompt injection aimed at review agents.
- **Security tiers 0/1/2:** tier 0 baseline; tier 1 adds data-handling note; tier 2 (keys/funds, wallets, AI agents) adds threat model, approval enforced in the core module, no plaintext keys, spend caps outside the agent, and **independent review/bug-bounty window before adoption counts**.
- **Adoption from signed releases:** installs, testimonials and third-party apps count only if using signed releases from the declared catalog on testnet; third-party apps pin the signer DID or `rootHash`.
- **Post-award:** signed releases only, no replaced/deleted assets, 72-hour vulnerability/compromise notice; prize may hold back part of the award (PR suggests 15–25%, undecided; `TERMS.md` not edited).
- **Evaluation wording:** "evaluated `rootHash` X on date Y against criteria Z"; not an approval/verification/audit/safety claim; "Logos never DMs install links".
- **Validation tooling:** non-blocking warnings for missing section/commit SHA/DID/`rootHash`/catalog URL, plus a read-only GitHub API check of release `immutable` status using `GH_TOKEN` with strict pattern matching on submitter strings.

**Review state:** #189's issue comment is only the validation bot failing the check (wrong title, touches files outside `solutions/`, placeholders) because the PR modifies repo policy files. There are **no inline review comments** on #189. No approval or merge. Its PR body lists six open decisions (scope in time, held-back percentage/period and `TERMS.md`, Tier 2 vs "release early" — explicitly noting conflict with LP-0023-style advice — rolling `index` vs immutability, base template SHA pins, when checks become blocking).

## 6. Exact changes that would affect the solution draft

**If PR #188 merges before submission (high impact on architecture):**
- SPEL-built registry must become a LEZ actor program using `actor_entrypoint!` with Borsh messages.
- IDLs must be `lez-idl`, generated from source and byte-for-byte reproducible; the registry's own IDL must be generated from source rather than by SPEL.
- Need tooling (own or reused) for IDL generation, typed message encode/decode of messages/state/events, call planning/simulation (actors, signers, apply/revert, cycles, fee), call sending, and `image_id` computation/comparison against the deployed header.
- SDK must decode messages/state/events with the IDL; 50 adoption entries' IDLs must parse as LEZ IDL.
- Note reviewer's 404 on the referenced IDL-format link and the SPEL question — verify the final merged wording before pivoting.

**If PR #189 merges with scope covering already-open prizes (high impact on packaging/adoption evidence):**
- Add `## Release Integrity` section to the solution with full manifest (commit SHA, catalog URL, `rootHash`, SHA-256, variants, `did:jwk:` signer DID, release URL); declare the DID up front in PR + README.
- Sign all `.lgx` releases with `lgx sign`, list DID in catalog `trustedSigners`, host catalog on a GitHub organisation, enable release immutability before publishing, ship only new versions (never mutate the evaluated build).
- Include exact canonical install instructions (catalog URL + DID + `rootHash`); no "search for" or warning-disable guidance.
- Pin third-party Actions to commit SHAs; 2FA; protected branch/tags; key not exposed to PR workflows; committed lockfiles; demo without `--accept-flake-config`/extra substituters/prompt injection.
- Determine the prize's Security Tier (LP-0023 on master currently has **no** `Security Tier` line) and ensure adoption evidence comes only from signed releases with third-party apps pinning DID or `rootHash`; Tier 2 would add independent-review delay before adoption counts.
- Expect eval wording by `rootHash` and possible held-back payment; `TERMS.md` has not been updated in the PR.

**Regardless of merge (master items to double-check in the draft):**
- Document positions on all three "Additional Design Choices" (on-chain data boundary, verified-source hosting, trusted-signer overlay).
- Satisfy the "No mandatory centralised external services" reliability criterion (disclosure/disable/opt-in analytics).
- Catalog fork via `logos-modules-release-base` + `logos-modules-release-action` and include the `logos-repo.json` URL; dual MIT + Apache-2.0 licensing.
- Clean-environment demo script, per-month adoption counts, and the SDK worked example.

## 7. Action needed

1. **Proceed with the solution PR** — there is no competing LP-0023 solution PR.
2. **Draft against current master** (SPEL + SPEL-generated IDL, testnet 0.4) because that is the published, evaluated spec today.
3. **Track #188 before opening the PR.** If it merges, pivot the registry to a LEZ actor program and implement/verify the `lez-idl` tooling list in §6; confirm the merged IDL-format URL works (reviewer reported 404).
4. **Track #189 before opening the PR.** It is a draft with unresolved scope; if it merges and applies to open prizes, add the Release Integrity manifest/signing/immutability work in §6. Ask maintainers for a scope/grace-date decision rather than assuming it applies to LP-0023.
5. **Re-run this check immediately before submitting**, since first-come-first-served evaluation makes a late spec merge consequential.

## 8. Explicitly NOT verified

- The exact date/snapshot of the "earlier research"; therefore §3's comparison is against the expectation list supplied in the task, not a byte-level diff of an earlier spec copy.
- Whether PR #188 or #189 will merge, when, or with what amendments — both are currently open and neither shows maintainer approval in the comments checked.
- Whether Logos testnet 0.4 is actually live/reachable; this check only establishes that the spec requires it.
- Whether any solution PR outside this repository targets LP-0023 (only `logos-co/lambda-prize` was searched).
- Any effect of #189 on `TERMS.md` or held-back payment percentages — the PR states these are undecided and `TERMS.md` was not edited.

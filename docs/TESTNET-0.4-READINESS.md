# Logos LEZ Testnet 0.4 Readiness — LP-0023 (`proclaim`)

**Research date:** 2026-11-19
**Subject:** Whether λPrize LP-0023 (LEZ Program Registry) can be deployed end-to-end on the Logos LEZ zone on **testnet 0.4**, and what would have to change.
**Method:** Official Logos docs (`docs.logos.co`), Logos roadmap (`roadmap.logos.co`), official Logos articles, official X account posts, `logos-blockchain/logos-execution-zone` releases/tags and commit API, `logos-co/spel` release commit, live HTTP/JSON-RPC probes, and read-only inspection of local project configs. No secrets, seeds, private keys, or credentials were printed or copied.

---

## 1. Definitive answer

**NOT LIVE — Logos testnet / LEZ zone 0.4 is not publicly reachable as of 2026-11-19.**

The earlier belief was **not stale**: it is confirmed by primary sources.

- The official Logos **“Testnet endpoints”** page lists the **0.4 (actor model)** row as **`TBD` for sequencer RPC, LEZ explorer, blockchain explorer, release, and channel ID**. The same table gives concrete values for 0.3 and 0.2.4: https://docs.logos.co/get-started/testnet-endpoints
- The newest `logos-execution-zone` release is **`v0.4.0-rc3`**, explicitly a **Pre-release**, published **11 Nov 2026** (previous RCs 29 Oct and 10 Nov). There is **no stable `v0.4.0` tag/release**: https://github.com/logos-blockchain/logos-execution-zone/releases and https://github.com/logos-blockchain/logos-execution-zone/tags
- The official roadmap page **“LEZ Core and App Unification: Testnet v0.4”** (updated 13 Nov 2026) says it **“Depends on the LEZ programs landing on Nov 30, 2026 (other team)”**: https://roadmap.logos.co/logoscore/roadmap/testnet_v0.4/lez_v0.4
- The official Logos Tech account says **“Anything built on 0.2 or 0.3 will need porting to Testnet 0.4”** and that **LEZ-legacy (0.2.4)** exists to keep building **“until Testnet 0.4”**: https://x.com/Logos_devs/status/2108733240211292642 and https://x.com/Logos_devs/status/2110463437356843176
- The official scaffold guide states: **“`logos-scaffold` cannot deploy to the current LEZ public testnet”** and directs developers to a standalone wallet; that warning is written for **0.3**, not 0.4: https://docs.logos.co/lez/programs/write-and-deploy-lez-program-with-scaffold

What **is** live and publicly reachable today:

| Network | Sequencer RPC | LEZ Explorer | Release |
| --- | --- | --- | --- |
| 0.4 (actor model) | **TBD (none published)** | **TBD (none published)** | **none; RC only** |
| 0.3 (current) | `https://testnet.lez.logos.co` | `https://explorer.testnet.lez.logos.co/` | `v0.3.2` |
| 0.2.4 (LEZ-legacy) | `https://testnet-legacy.lez.logos.co/` | `https://explorer.testnet-legacy.lez.logos.co/` | `v0.2.4-lez-legacy` |

Live probes performed on 2026-11-19:

- `POST https://testnet.lez.logos.co/` with `getChannelId` → **HTTP 200**, result `0101010101010101010101010101010101010101010101010101010101010101`.
- `POST https://testnet-legacy.lez.logos.co/` with `getChannelId` → **HTTP 200**, result `0240240240240240240240240240240240240240240240240240240240240240`.
- Both explorer URLs → **HTTP 200**.

**Conclusion for LP-0023:** the master-spec requirement “deploy end-to-end on testnet 0.4” **cannot be satisfied today**. The only currently deployable public targets are **0.3** (requires porting from our 0.2.4 programming model) and **0.2.4 LEZ-legacy** (matches our current program model). Do not claim a 0.4 deployment until an official 0.4 endpoint, release tag, and explorer/API are published.

---

## 2. Evidence URLs (exact)

**Primary — 0.4 not public**

1. https://docs.logos.co/get-started/testnet-endpoints — endpoint matrix; 0.4 row is all `TBD`; 0.3 and 0.2.4 rows fully populated.
2. https://github.com/logos-blockchain/logos-execution-zone/releases — `v0.4.0-rc3` (11 Nov), `v0.4.0-rc2` (10 Nov), `v0.4.0-rc1` (29 Oct) are **Pre-release**; `v0.3.2` is marked **Latest**.
3. https://github.com/logos-blockchain/logos-execution-zone/tags — no `v0.4.0` final tag; newest tags are the three RCs.
4. https://github.com/logos-blockchain/logos-execution-zone/releases/tag/v0.4.0-rc3 — RC notes; bumps logos-blockchain to `0.3.5-rc.1`, described in the tag commit message as “the release the devnet runs”.
5. https://roadmap.logos.co/logoscore/roadmap/testnet_v0.4/lez_v0.4 — “Depends on the LEZ programs landing on Nov 30, 2026 (other team)”.
6. https://roadmap.logos.co/logoscore/roadmap/testnet_v0.4/ — 8 v0.4 work items, all dated 13 Nov 2026; roadmap folder exists, no launch announcement.
7. https://x.com/Logos_devs/status/2110463437356843176 — “LEZ-legacy is live… until Testnet 0.4”; reply promised RPC/explorer/release notes.
8. https://x.com/Logos_devs/status/2108733240211292642 — “Testnet 0.4 is the target. It will break things…”
9. https://logos.co/media/article/logos-testnet-v03-live — v0.3 went live **30 Sep 2026**; explicitly says **smaller breaking changes possible in Testnet v0.4**.

**Connection / tooling / explorer**

10. https://docs.logos.co/lez/get-started/run-lez-wallet-via-cli — `wallet change-network`, `wallet check-health`, `wallet deploy-program` in the wallet-command reference; target `https://testnet.lez.logos.co`.
11. https://docs.logos.co/lez/programs/write-and-deploy-lez-program-with-scaffold — localnet-only warning, `LEE_WALLET_HOME_DIR` / `NSSA_WALLET_HOME_DIR`, `logos-scaffold deploy`, port `3040` caveat.
12. https://docs.logos.co/lez/extensions/admin-authority — exact legacy deployment recipe: `cargo install --git … --tag v0.2.4 wallet`, `export LEE_WALLET_HOME_DIR=~/lez-wallet`, `wallet change-network https://testnet-legacy.lez.logos.co`, `wallet check-health`, `wallet deploy-program <bin>`; SPEL `v0.7.0` / LEZ `v0.2.4` pins; rustc `1.94.0+` floor for the `spel` CLI.
13. https://docs.logos.co/lez/follow-chain/run-lez-indexer — indexer JSON-RPC on `http://localhost:8779`; methods `getAccount`, `getLastFinalizedBlockId`, `checkHealth`; `getChannelId` curl against the public 0.3 sequencer.
14. https://docs.logos.co/lez/follow-chain/start-and-use-instance-of-lez-explorer — public LEZ explorer link; block/transaction/account search.
15. https://github.com/logos-blockchain/logos-execution-zone/releases/tag/v0.2.4-lez-legacy — legacy rebuild against logos-blockchain/zone-sdk 0.3.1, Rust 1.98.1, Docker tag `lez-legacy`.
16. https://api.github.com/repos/logos-blockchain/logos-execution-zone/commits/47eba256479f6f785acbd138834340703cd03401 — our local LEZ pin is the **“Merge dev to main for v0.2.4”** commit (7 Aug 2026).
17. https://api.github.com/repos/logos-co/spel/commits/4107dcb14ec0eb64e9e80a9691a3df0b9acd3c2b — our local SPEL pin is the **release v0.7.0** commit (17 Sep 2026); its changelog confirms LEZ v0.2.4 and the `sequencers: [{ sequencer_addr }]` config shape.

---

## 3. Connection details and conventions

### 3.1 Testnet 0.4

**None published.** No sequencer/RPC URL, chain/program-ID convention document, faucet, explorer, indexer API, or release tag. The official endpoint table says `TBD`.

### 3.2 Published values for the two live networks (reference only — not 0.4)

| Field | 0.3 (current) | 0.2.4 (LEZ-legacy) |
| --- | --- | --- |
| LEZ sequencer RPC | `https://testnet.lez.logos.co` | `https://testnet-legacy.lez.logos.co/` |
| LEZ explorer | `https://explorer.testnet.lez.logos.co/` | `https://explorer.testnet-legacy.lez.logos.co/` |
| Blockchain explorer | `https://testnet.blockchain.logos.co/web/explorer/` | `https://testnet.blockchain.logos.co/web/explorer/` |
| Release to build from | `v0.3.2` | `v0.2.4-lez-legacy` |
| Channel ID (verified by live `getChannelId`) | `0101010101010101010101010101010101010101010101010101010101010101` | `0240240240240240240240240240240240240240240240240240240240240240` |

**Faucet / funding:** the 0.3 wallet guide says to fund by **bridging from Logos Blockchain with a channel deposit** (paste the account ID without the `Public/` prefix) **or receiving a transfer from another testnet user**; it does not publish an LEZ faucet URL. Older LEZ/scaffold material mentions a **Piñata faucet** on the legacy local/net flow. Logos Blockchain documentation describes a public testnet faucet but the LEZ docs consulted here do not give a canonical faucet URL. **No 0.4 faucet is documented — do not invent one.**

### 3.3 Wallet files and commands

- Wallet home is selected by `LEE_WALLET_HOME_DIR` (LEZ v0.2.0+); `NSSA_WALLET_HOME_DIR` was used by v0.1.2.
- `wallet_config.json` must use the array form (SPEL v0.7.0 release note: the old flat `sequencer_addr` key is **silently ignored**):

  ```json
  {
    "sequencers": [{ "sequencer_addr": "<OFFICIAL_SEQUENCER_URL>" }],
    "seq_poll_timeout": "30s",
    "seq_tx_poll_max_blocks": 15,
    "seq_poll_max_retries": 10,
    "seq_block_poll_max_amount": 100,
    "calibration_limit": 100
  }
  ```

  Our local file currently has `http://127.0.0.1:3040` as the only sequencer (public value only; no credentials present).
- `storage.json` is the wallet’s persistent, password-seeded key material/local state. On a fresh wallet home, `wallet change-network <url>` initialises it, asks for a password, and prints a recovery phrase. **Never copy, print, or reuse a localnet `storage.json` for a public network.** In the 0.4 line, release notes indicate wallet storage encryption uses **argon2** with the password (v0.4.0-rc1, PR #987), so wallet binaries from different release lines are not interchangeable.
- Core commands: `wallet change-network <url>`, `wallet check-health`, `wallet account new public`, `wallet auth-transfer init --account-id Public/<id>` (needed before a key first signs for a SPEL program), `wallet deploy-program <bin>`, `wallet account get --account-id Public/<id>`.
- `lgs` is the short alias of `logos-scaffold`; our Makefile calls `wallet deploy-program $(PROGRAM_BIN)` directly, and our docs use `lgs spel -- …` / `lgs wallet -- …` wrappers.

### 3.4 Program / account ID conventions

- **Program ID** is the RISC Zero image ID computed from the **R0BF-wrapped `.bin`**, not from the raw ELF. Derive with `spel program-id <…/registry.bin>` (or `lgs spel -- program-id … --format hex`); output is a 32-byte hex value. The current localnet ID is `4df7d5396b68a16aae6d0f768ef447a0055eb43866827154f71139e486265144` (LP0023.md) and is **localnet-only** — a rebuilt/port program gets a new ID.
- **Account IDs** are base58; the wallet CLI prints them with a `Public/` prefix while raw CLI arguments often take the bare ID. **PDA** addresses are derived from `(program_id, seed)` / program-specific seeds via `spel pda` or `lgs spel -- pda …`.
- On 0.4, expect these surfaces to move: the v0.3 line already introduced **incremental account diffs**; v0.4 RCs add **pptx fees with dry-runs/fee params**, **FFI primitive unification**, **sequencer address option changes**, **argon2 wallet storage**, and cross-zone changes. A 0.2.4 SPEL program is **not** source- or ID-compatible with 0.4.

---

## 4. Public explorer / API for verification

- **0.4:** none published.
- **0.3:** public LEZ Explorer `https://explorer.testnet.lez.logos.co/` (verified HTTP 200). Docs say it supports browsing blocks and searching transactions and accounts. Blockchain-side explorer: `https://testnet.blockchain.logos.co/web/explorer/`.
- **0.2.4 legacy:** public LEZ Explorer `https://explorer.testnet-legacy.lez.logos.co/` (verified HTTP 200).
- **Indexer JSON-RPC:** the LEZ Indexer serves HTTP/WebSocket JSON-RPC on `http://localhost:8779` by default with `getAccount`, `getLastFinalizedBlockId`, `checkHealth`, and event methods. The official docs do **not** publish a public hosted indexer RPC URL; only the sequencer URL is public. For verification of a deployed program on a public network today, use the relevant explorer plus `wallet account get --account-id Public/<id>` / the wallet’s `chain-info` commands.
- Practical caveat from the official 0.4-adjacent docs: on the legacy testnet, blocks are roughly a minute apart; `spel` may report `Transaction NOT confirmed` or `wallet` `All pollers failed` even when a transaction later lands. Check the signer key’s `nonce` before resending.

---

## 5. What must change in our localnet workflow to target 0.4

Read-only snapshot of local files (public/non-secret values only):

- `spel.toml`: `idl = "registry-idl.json"`, `binary = "methods/guest/target/riscv32im-risc0-zkvm-elf/docker/registry.bin"`.
- `scaffold.toml`: LEZ pin `47eba256…` (= v0.2.4 merge commit), SPEL pin `4107dcb1…` (= v0.7.0 release), `framework.kind = "spel"`, `framework.version = "0.1.0"`, `[build] guest = "local"`, `[localnet] port = 3041`, `risc0_dev_mode = true`, `[circuits] version = "0.4.2"`, `[wallet] home_dir = ".scaffold/wallet"`.
- `.scaffold/wallet/wallet_config.json`: single sequencer at `http://127.0.0.1:3040` plus poll settings (no credentials). `storage.json` and `statistics.json` exist and were **not** inspected or copied.
- `LP0023.md`: localnet-complete; program ID, PDAs, author/verifier account IDs, tx hashes and CIDs are localnet values recorded there.

Required changes once (and only once) an official 0.4 endpoint exists:

1. **Dependency pins:** replace the v0.2.4 LEZ pin and SPEL v0.7.0 pin in `scaffold.toml` (and any `Cargo.toml`/`Cargo.lock` pins, including the `nssa_core`/`lee_core` alias and `spel-framework`) with the **official 0.4 release tag/commits**. Do not point at a moving `main` branch. `[circuits] version = "0.4.2"` must be reconciled with whatever circuits release the chosen 0.4 pin uses.
2. **Port the guest:** adapt the six-instruction registry to the 0.4 LEE state-diff/fees API; regenerate the IDL (`lgs build idl` / `spel generate-idl`), rebuild the R0BF `.bin`, and recompute the program ID. Expect the IDL/ABI and PDA derivations to change; re-run `init-index`, `register`, `mark-verified` with the **new** program ID and fresh evidence CIDs.
3. **Proof mode:** set `[localnet] risc0_dev_mode = false` (or stop using localnet for the public target) and do **not** rely on dev-mode proofs. Currently dev mode is `true` and `[build] guest = "local"`; for reproducible IDs prefer the pinned Docker guest build (`guest = "docker"`) if Docker becomes available, otherwise document that a local-built ID may differ across machines.
4. **Wallet home:** use a **fresh** `LEE_WALLET_HOME_DIR` pointing at a new directory; write a `wallet_config.json` with the official 0.4 sequencer in the `sequencers` array; run `wallet change-network <official-0.4-RPC>` and `wallet check-health`; set a new password and back up the recovery phrase securely. Do **not** reuse `.scaffold/wallet/storage.json`.
5. **Makefile / commands:** `make deploy` uses `wallet deploy-program $(PROGRAM_BIN)`. Install the `wallet` binary from the **same 0.4 release tag** as the guest dependency (`cargo install --git https://github.com/logos-blockchain/logos-execution-zone.git --tag <v0.4.x> wallet`). `lgs spel` must be built from the matching SPEL release, otherwise `spel program-id`/`pda`/`inspect` can disagree with the chain.
6. **Funding:** plan a funded account on the target chain. No 0.4 faucet is published; funding may require a blockchain→zone bridge deposit or another testnet user.
7. **Verification path:** add checks against the official 0.4 explorer/API once published; until then, a 0.4 claim is unverifiable by third parties.
8. **Port caveat:** scaffold’s `[localnet] port = 3041` only affects project bookkeeping — official docs note the local sequencer always binds `3040` and that changing the port does not redirect the wallet. For a public chain, the URL in `wallet_config.json` is what matters.

---

## 6. Exact next commands

### 6.1 For testnet 0.4 — **blocked today**

There are **no valid 0.4 commands to run** because no endpoint, release tag, or explorer exists. Do **not** guess hostnames or ports. The first actionable command is a re-check of the official endpoint table and release page:

```bash
# Re-check authority (not a deployment):
# https://docs.logos.co/get-started/testnet-endpoints
# https://github.com/logos-blockchain/logos-execution-zone/releases
```

Once (and only once) Logos publishes an official 0.4 sequencer URL and final release tag, the sequence will be:

```bash
# Placeholders MUST be replaced with officially published values.
export LEZ_04_REPO=https://github.com/logos-blockchain/logos-execution-zone.git
export LEZ_04_TAG=<OFFICIAL_v0.4_STABLE_TAG>
export LEZ_04_RPC=<OFFICIAL_0.4_SEQUENCER_URL>
export LEE_WALLET_HOME_DIR=$HOME/lez-wallet-04
mkdir -p "$LEE_WALLET_HOME_DIR"

git clone "$LEZ_04_REPO" "$HOME/logos-execution-zone-04"
cd "$HOME/logos-execution-zone-04" && git checkout "$LEZ_04_TAG"
cargo install --git "$LEZ_04_REPO" --tag "$LEZ_04_TAG" wallet

# Write $LEE_WALLET_HOME_DIR/wallet_config.json with:
# {"sequencers":[{"sequencer_addr":"<OFFICIAL_0.4_SEQUENCER_URL>"}], ...}
wallet change-network "$LEZ_04_RPC"
wallet check-health

# From the ported proclaim project: rebuild IDL + guest, derive ID, deploy.
lgs build idl
cargo +risc0 build --release --manifest-path methods/guest/Cargo.toml \
  --target riscv32im-risc0-zkvm-elf
# wrap ELF into R0BF .bin (as documented in LP0023.md), then:
lgs spel -- program-id methods/guest/target/riscv32im-risc0-zkvm-elf/docker/registry.bin --format hex
wallet deploy-program methods/guest/target/riscv32im-risc0-zkvm-elf/docker/registry.bin
```

Then re-run the full lifecycle (`init-index`, `register`, `lookup`, `search`, independent `mark-verified`, `inspect`) with the newly derived program ID and new account IDs, and verify via the published 0.4 explorer/API.

### 6.2 What can be done today (alternatives — **not** 0.4)

**A. LEZ-legacy 0.2.4 — matches our current SPEL v0.7.0 / LEZ v0.2.4 program:**

```bash
cargo install --git https://github.com/logos-blockchain/logos-execution-zone.git --tag v0.2.4 wallet
export LEE_WALLET_HOME_DIR=$HOME/lez-wallet-legacy
mkdir -p "$LEE_WALLET_HOME_DIR"
wallet change-network https://testnet-legacy.lez.logos.co
wallet check-health
wallet deploy-program methods/guest/target/riscv32im-risc0-zkvm-elf/docker/registry.bin
# Verify: https://explorer.testnet-legacy.lez.logos.co/
```

**B. LEZ 0.3 — current public testnet, but requires porting from our 0.2.4 model:**

```bash
git clone https://github.com/logos-blockchain/logos-execution-zone.git
cd logos-execution-zone && git checkout v0.3.2
cargo install --path lez/wallet --force
wallet change-network testnet      # configures https://testnet.lez.logos.co
wallet check-health
# Port + rebuild the guest against the 0.3 API before deploying.
wallet deploy-program <newly-built-0.3-bin>
# Verify: https://explorer.testnet.lez.logos.co/
```

Neither A nor B satisfies a literal “testnet 0.4” requirement; treat them as interim evidence only and label them explicitly.

---

## 7. Blockers

1. **No public 0.4 network:** RPC, explorer, release, and channel all `TBD` in official docs.
2. **No stable release:** only `v0.4.0-rc1/2/3` pre-releases; `v0.3.2` is still marked “Latest”.
3. **Upstream dependency:** official roadmap says LEZ programs land **30 Nov 2026**, after today.
4. **Breaking programming model:** incremental diffs (v0.3) plus v0.4 RC changes (fees/dry-runs, FFI unification, argon2 storage, cross-zone) mean our SPEL v0.7.0 / LEZ v0.2.4 program must be ported and re-audited; program ID and all PDAs change.
5. **No published 0.4 verification surface:** cannot prove a deployment to an independent reviewer.
6. **Funding unknown:** no published 0.4 faucet; bridge/transfer path needs confirmation after launch.
7. **Local toolchain mismatch:** local pins are v0.2.4/v0.7.0 with circuits 0.4.2, dev-mode proofs enabled, and local-built guest artifacts; these are not 0.4-compatible.

**Recommended decision:** block the 0.4 submission milestone; keep the localnet result as-is, optionally publish a clearly-labelled legacy-0.2.4 or ported-0.3 deployment as interim evidence, and re-run this readiness check after 30 Nov 2026.

---

## 8. Unverified / do not assume

- Whether an **invite-only or internal 0.4 devnet** exists. The v0.4.0-rc3 tag message mentions “the release the devnet runs”, but **no public endpoint/credentials are published**; the public docs still say `TBD`.
- The **exact 0.4 launch date**. “Targeting end of November” and the Nov 30 dependency are not a launch announcement. (A YouTube update was titled “What’s Coming in Logos Testnet v0.4”, i.e. forward-looking.)
- Whether λPrize judges will **accept 0.2.4-legacy or 0.3** in place of 0.4, or grant a deadline extension. No public λPrize ruling found.
- Whether a **public 0.4 faucet** will exist, and whether bridge deposits will be enabled at launch.
- Any **Discord-only** endpoint, credentials, or launch claim; no public Discord-derived document was found that supplies a 0.4 URL.
- Whether the hosted 0.3/legacy explorers expose a **public indexer JSON-RPC** equivalent to `localhost:8779`; docs only publish the explorer UI and sequencer URL.
- Exact hardware/toolchain requirements for building the 0.4 wallet/CLI on this Windows/WSL machine; LEZ legacy docs cite Rust 1.98.1 and SPEL CLI 1.94.0+, but the 0.4 line uses its own pinned toolchain and has not been built here.

---

## 9. Safety note

Only public configuration values were read. `.scaffold/wallet/storage.json` (34,920 bytes, key material) was **not opened, printed, or copied**. `wallet_config.json` was inspected with key-like/long values redacted; it contained only the localhost sequencer URL and polling settings. No tokens, seeds, passwords, or private keys are included in this report.

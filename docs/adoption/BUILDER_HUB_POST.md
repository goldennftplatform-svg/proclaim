<!-- SPDX-License-Identifier: MIT AND Apache-2.0 -->

# Draft post for Logos Discord — #builder-hub

> Copy/paste ready. Post from your own account, and edit the placeholders in
> `<...>` before posting. This is an **early draft** announcement: it does not
> claim any prize criterion as met, and there is no adoption to report yet.

---

**Subject: Early draft: LEZ Program Registry (LP-0023) — looking for independent UI/module authors to integrate**

Hey #builder-hub 👋

I've published an **early draft** of `proclaim`, a LEZ Program Registry built
with SPEL, for Logos λPrize LP-0023. Repo (MIT AND Apache-2.0):
https://github.com/goldennftplatform-svg/proclaim

**Where it stands, honestly:**
- The full lifecycle works **on localnet only** so far: deploy → register →
  search → independent `mark_verified` → inspect. Self-verification and
  unauthorized updates are rejected on-chain.
- **Testnet 0.4 is pending** — there is no testnet deployment yet.
- The Qt/QML UI is a **scaffold**, and the installable `.lgx` package is
  **pending**. Cycle counts are open, and CI currently covers host-side checks
  only.
- **No adoption yet.** I'm not claiming any of the adoption criteria are met.

**What a contributor gets to build against:**
- On-chain program with 6 instructions, one PDA per program ID, and a global index.
- Rust SDK `registry_core` (types, PDA helpers, search filters) + generated C FFI header.
- Read-only HTTP API on `127.0.0.1:3030` and a dependency-free JS/TS client
  (`@lp0023/registry-client`). No centralized service required beyond the
  configured sequencer and Logos Storage.

If you have a LEZ program or a UI module, there's a copy-paste
**15-minute integration guide** here:
https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/INTEGRATION.md

**Concrete asks (any one of these helps a lot):**
1. **Register your own program** in the local registry (or tell me what's
   confusing about the guide) so we have more than one independent entry.
2. **Run the independent source verifier** on your own program from a different
   account — `scripts/verify-source.sh` — and share what broke or was unclear.
   A second pair of eyes on the rebuild path is genuinely useful.
3. **Integrate the client into a UI module** (even a tiny read-only search view)
   and tell me whether the HTTP API / JS client / FFI surface fits your needs.
   I'll help debug integration issues in this thread.

Feedback on the API shape is welcome now — it's early enough to change. I'll
post updates here as testnet 0.4 and packaging land.

---

### Internal note (not for posting)

- Do not imply 5 third-party modules, 50 programs, verified-entry volume, or
  testimonials exist. Current honest counts are in `ADOPTION_TRACKER.md`.
- Do not present the registry as deployed on testnet or as an installable
  Basecamp module.
- Keep the tone peer-to-peer: asking for integration help, not announcing
  a finished product.

<!-- SPDX-License-Identifier: MIT AND Apache-2.0 -->

# Public testnet deploy validation — result (2026-11-20)

## Verdict

**No public deployment was produced.** There is no program address, transaction hash, block ID, or explorer receipt to claim. The registry guest and the full lifecycle remain verified only against a fresh local standalone sequencer with `RISC0_DEV_MODE=1`.

The live Logos testnet accepted wallet setup, balance reads and IDL generation, and a correctly packaged R0BF binary reached segment upload. It then stopped on three client/server compatibility gaps between the tested LEZ v0.4.0-rc3 toolchain and the live zone.

## What was observed

1. **Wallet, balance and IDL gates passed.** A fresh isolated wallet was created, funded genesis accounts were imported, the payer balance was read from the live endpoint, and the registry IDL was generated from the guest core crate.

2. **Raw ELF was rejected.** Submitting the raw guest ELF returned `Program bytecode is not a valid RISC0 program binary` / `Malformed ProgramBinary`. Packaging the same ELF with `ProgramBinary::new(user_elf, V1COMPAT_ELF).encode()` produced a 372,500-byte R0BF `.bin` that passed the format gate and entered segment upload.

3. **Account-data schema skew.** Live `getAccountView` returns `data.shards`; the RC3 client expects `actor_states`. A local serde alias resolved the parse error for diagnostics only. It did not make the transaction format compatible.

4. **`getProofsAndRoot` shape skew.** Live returns a 2-element result `[proofs, root]`; the RC3 wallet expects `[proofs, message_paths, root]`. The mismatch produced `invalid type: integer 191, expected a tuple of size 2`. A temporary local RPC shim rewrote the response to `[proofs, [], root]` so the deploy could advance to the next gate. This was a diagnostic instrument, not a product fix, and it has been stopped.

5. **`sendTransaction` shape skew.** With the proof-shape shim in place, the deploy reached `sendTransaction`. The RC3 wallet sent a base64-looking single-string transaction parameter; the live sequencer rejected it with JSON-RPC `-32602 Invalid params`, data `Unexpected length of input`. Segment upload stopped there. The captured request/response is kept locally; it is not published because the local probe log contains imported developer key material.

## Why the attempt stopped

Swapping in a different CLI is not a drop-in fix. The live program model exposes `program_loader` primitives such as `write_segment` and `create_header` and uses the current `PdaSeed` / `for_public_pda` account model. The registry guest was written against the LEZ v0.4.0-rc3 actor entrypoint API (`ReceiveInput`, `Response`, `SyncMessage`, `actor_entrypoint`). A compatible client alone would not make this guest deployable, and forcing a transaction through compatibility shims would not validate the real product.

Porting the guest and deploy tooling to the current LEZ program model is a separate engineering task. It has not been done.

## State left behind

- `PROGRAM_ADDRESS=` is empty in every deploy run.
- No transaction hash, block ID, or explorer receipt exists for this work.
- Failed segment uploads may have left orphaned header/segment accounts on the testnet. They are not a deployed program.
- The temporary local RPC proxy has been stopped.
- No solution PR was opened.

## Evidence

- Public local lifecycle receipts: `E2E-RESULT.md` at https://github.com/goldennftplatform-svg/proclaim/blob/master/E2E-RESULT.md
- Public status: https://goldennftplatform-svg.github.io/proclaim-site/brief.html
- Local, unpublished diagnostics: `registry-rc3/live-evidence/deploy.log`, `deploy-proxy.log`, and `rpc-proxy.log`. These contain imported developer key material and are deliberately not linked as public evidence.
- Live endpoint probed: `https://testnet.lez.logos.co/`; explorer: `https://explorer.testnet.lez.logos.co/`.

## Required before any deployment claim

1. Port the registry guest and deploy path from LEZ v0.4.0-rc3 to the live LEZ program model.
2. Rebuild the guest and CLI with `CARGO_BUILD_JOBS=2` and rerun the lifecycle against a local sequencer.
3. Retry the live deploy without compatibility shims.
4. Present a real program address, transaction hash, block ID, and explorer receipt. Until all four exist, there is no deployment to report.

## Scope

This is a deployment compatibility result. It does not add prize credit, third-party adoption, users, testimonials, real-proving evidence, or a Basecamp launch success.

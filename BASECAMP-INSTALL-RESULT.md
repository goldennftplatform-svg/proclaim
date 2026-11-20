<!-- SPDX-License-Identifier: MIT AND Apache-2.0 -->

# Basecamp install test — `registry.lgx`

**Date:** 2026-11-20  
**Result:** **PASS (local install only)** — the signed `registry.lgx` UI module installed into a real Logos Basecamp 0.4.0-rc.2 AppImage using the Package Manager UI. Basecamp's own details panel reports `Status: Installed`, version `0.1.0`, and the expected root hash.

This is a developer-run install check on one Linux machine. It is **not** third-party adoption, not a deployed/testnet result, and not prize-criterion credit. No users, testimonials, or independent modules were created.

---

## What was tested

| Item | Value |
| --- | --- |
| Package | `/home/xps/logos-work/registry/ui/registry/registry.lgx` |
| SHA-256 | `1024036067711c06fac158be891422b362c71b7ebeefadcc1dd6c389d113c59e` |
| Size | 74,377,042 bytes |
| Manifest name/version | `registry` `0.1.0`, type `ui`, category `tools`, manifestVersion `0.7.0` |
| Variant installed | `linux-amd64` → `libregistry_plugin.so` |
| Basecamp build | `logos-basecamp-x86_64.AppImage` from `logos-co/logos-basecamp` release `pre-release-4cea4bb-1433` (0.4.0-rc.2) |
| Session | Isolated `--user-dir /home/xps/logos-work/basecamp-test/session` |
| Platform | WSL2 / Ubuntu x86_64, X11 (`QT_QPA_PLATFORM=xcb`), software Qt Quick backend |

The user's pre-existing Basecamp instance (default data dir, running a node) was left running and untouched. Only the isolated test process was used; it has been stopped after evidence capture.

---

## Exact steps

1. Downloaded the real release binary:

   ```bash
   mkdir -p /home/xps/logos-work/basecamp-test
   gh release download pre-release-4cea4bb-1433 \
     --repo logos-co/logos-basecamp \
     --pattern 'logos-basecamp-x86_64.AppImage' \
     --dir /home/xps/logos-work/basecamp-test --clobber
   ```

2. Confirmed the package itself is well formed (this does **not** mean the signer is trusted):

   ```bash
   /home/xps/logos-work/tools/logos-package/build/lgx verify \
     /home/xps/logos-work/registry/ui/registry/registry.lgx
   ```

   Output: structure valid, signature valid, signer DID `did:jwk:eyJjcnYiOiJFZDI1NTE5…`, self-asserted name `proclaim LP-0023`, and **"Signer DID is NOT in trusted keyring"**. Full output in `basecamp-install-evidence/lgx-verify.txt`.

3. Started Basecamp with an isolated session and file logging:

   ```bash
   export DISPLAY=:0
   env -u WAYLAND_DISPLAY DISPLAY=:0 \
     QT_QPA_PLATFORM=xcb QT_QUICK_BACKEND=software \
     QSG_RHI_BACKEND=software LIBGL_ALWAYS_SOFTWARE=1 \
     /home/xps/logos-work/basecamp-test/logos-basecamp-x86_64.AppImage \
     --user-dir /home/xps/logos-work/basecamp-test/session
   ```

4. In the UI: **Manage Packages → Install Local Package**, then entered the absolute path in the **Select LGX Package to Install** dialog:

   ```text
   /home/xps/logos-work/registry/ui/registry/registry.lgx
   ```

5. Confirmed the trust warning and install. Basecamp logged the untrusted-key warning and then the extracted plugin path:

   ```text
   [2026-11-20 15:27:45.638] [warning] [logos] [package_manager]
     Warning: Package is signed by a key no trusted anchor validates:
     /tmp/pmui-DqtgfF/cd803076-2f42-4dfc-8eb4-6482ede6c4a9.lgx
   [2026-11-20 15:27:45.642] [info] [logos] [package_manager]
     signer claims to be: proclaim LP-0023 (self-asserted, not verified)
   UI plugin file installed:
     "/home/xps/logos-work/basecamp-test/session/plugins/registry/libregistry_plugin.so"
   ```

6. Reopened the package details in the UI. The panel reads:

   ```text
   Package: registry
   Type: ui
   Category: tools
   Repository: local
   Status: Installed
   Installed version: 0.1.0
   Installed hash: 4979d0f4992d9678cb5b07efc9d12b22938a59bbed0f42f471d5cc477fd53f50
   Dependencies: None
   ```

---

## Installed files

Under `session/plugins/registry/`:

| File | Size (bytes) | SHA-256 |
| --- | ---: | --- |
| `Main.qml` | 193,197 | `6e8a9b917d9031681d0dce846cf4cafaae5f32de5e90d65fa9bfbf5a26dbe5af` |
| `libregistry_ffi.so` | 313,670,152 | `5b949fc50b52cadd1b680cd69fd4978b7cc4af1ab1714a30546b01d7d5a6aed7` |
| `libregistry_plugin.so` | 191,136 | `ed31822d994f7a7e1c880be95e1ef91f9851da126a2e8cc5dbeba1213b90eb09` |
| `manifest.json` | 573 | `02672b3bb6482dc394c79bc4f2ce24d00a2cebec95d55adcfc4164db387cb581` |
| `manifest.sig` | 412 | `3f0cc29886736aad72a014f6485a050b39b7fec8fb197066c6728dfe99d4e466` |
| `variant` | 11 | `abb35c616421af72198ad7c2aeeef38516f08f6a7afb2a728cf0068a8a712ddc` |
| `.lgpm-source` | 64 | points to the temporary local LGX copy created by the Package Manager |

`manifest.json` root hash `4979d0…3f50` matches the hash shown as **Installed hash** in Basecamp's details panel.

---

## Evidence files

All relative to [`basecamp-install-evidence/`](basecamp-install-evidence/):

- `01-package-manager-after-install.png` — package list after install
- `02-tools-category.png` — `Tools` category filtered to `registry`
- `03-registry-details-installed.png` and `03-registry-details-ocr.txt` — UI details: `Status: Installed`, version, hash
- `04-file-dialog-path-entered.png` and `04-file-dialog-ocr.txt` — absolute LGX path in the installer dialog
- `05-basecamp-welcome.png` — Basecamp starting in the isolated session
- `install-log-excerpt.txt` — raw timestamped trust warning and install line
- `lgx-verify.txt` — `lgx verify` output
- `sha256.txt` — source and installed-file hashes

Screenshots are X11 window captures; OCR transcripts are included because the capture is dark-themed. Both are local artifacts, not third-party attestation.

---

## Caveats / what this does **not** prove

- The signing key is **valid but untrusted**: Basecamp warned that no trusted anchor validates it. Installing required accepting that warning. The signer name/URL are self-asserted.
- Only the **linux-amd64** variant was installed. No macOS/Windows evidence.
- **Load/launch was not verified** in this test. Installation and UI recognition are confirmed; starting the widget is a separate check.
- This was done by the developer on the developer's machine. It is **not** an independent third-party module adoption and counts for zero prize-adoption items.
- No on-chain transaction, program deployment, proof, or testnet result is involved.
- The isolated test session was used specifically so the user's running node instance was not modified.

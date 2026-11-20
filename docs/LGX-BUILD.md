# Building and verifying the signed `registry.lgx` (local, no Docker/Nix)

This records the exact, reproducible procedure used to build and cryptographically
sign the Basecamp UI module package on 2026-11-19, and the verification output.
It is **not** the `logos-module-builder` (`mkLogosModule` / `mkLogosQmlModule`) Nix
path required by the literal prize wording — see the open question below.

## Prerequisites (Linux / WSL Ubuntu, installed with apt)

```
build-essential cmake git python3
qt6-base-dev qt6-declarative-dev zlib1g-dev libicu-dev libsodium-dev nlohmann-json3-dev
Rust toolchain (rustup), risc0 toolchain (`cargo +risc0`)
```

`lgx` is built from source at `https://github.com/logos-co/logos-package` with
`cmake -S . -B build -DCMAKE_BUILD_TYPE=Release && cmake --build build --parallel 2`.
System `nlohmann-json3-dev` avoids the CMake `FetchContent` GitHub clone that can
stall in restricted networks. `cpp-semver` is still fetched once (small).

## Build + sign (authoritative script: `wsl-lgx-build.sh` in the local work area)

```bash
source ~/.cargo/env
export RUSTFLAGS='--cfg getrandom_backend="custom"'
export CARGO_BUILD_JOBS=2
export PATH="/home/xps/logos-work/tools/logos-package/build:$PATH"

# 1. FFI (no IDL regeneration - generated artifacts are committed)
cargo build -p registry_ffi

# 2. Qt plugin + preview app
cmake -S ui/registry -B ui/registry/build -DCMAKE_BUILD_TYPE=Release
cmake --build ui/registry/build --parallel 2

# 3. Package (lgx add computes the manifest hashes over variants/ contents)
VARIANT="linux-amd64"
STAGE=ui/registry/.lgx-staging
mkdir -p "$STAGE"
cp ui/registry/build/libregistry_plugin.so \
   target/debug/libregistry_ffi.so \
   ui/registry/qml/Main.qml "$STAGE/"
cd ui/registry
lgx create registry
lgx add registry.lgx -v "$VARIANT" -f "$STAGE" -m libregistry_plugin.so -y

# 4. Merge curated metadata WITHOUT dropping generated hashes
python3 ../../scripts/patch_lgx_manifest.py registry.lgx manifest.json
lgx verify registry.lgx

# 5. Generate a dev signing key (first run only) and sign
lgx keygen --name lp0023-dev            # writes $HOME/.config/logos/keys/lp0023-dev.{jwk,pub,did}
lgx sign registry.lgx --key lp0023-dev \
  --name "proclaim LP-0023" \
  --url "https://github.com/goldennftplatform-svg/proclaim"
lgx verify registry.lgx

# 6. Inspect
lgx manifest registry.lgx
lgx signature registry.lgx
tar -tzf registry.lgx
```

### Why the manifest patch exists

`lgx add` writes a `hashes` map (Merkle root over `variants/` file bytes) into the
manifest. The curated `manifest.json` supplies descriptive fields (`type`,
`description`, `category`, `version`) but has no hashes. `scripts/patch_lgx_manifest.py`
therefore **merges** curated fields into the generated manifest and preserves
`hashes`, `manifestVersion` and `main`; replacing the whole manifest makes
`lgx verify` fail with `Missing content hashes in manifest`. No variant file bytes
change, so the preserved hashes remain valid.

## Verified result (2026-11-19)

```
$ lgx verify registry.lgx
Package structure is valid: registry.lgx
Signature is valid
Signer DID: did:jwk:eyJjcnYiOiJFZDI1NTE5Iiwia3R5IjoiT0tQIiwieCI6ImpiZDM1NHFnMkhHSkg4NVpHcVFJNmFjckNfQWs0SXE0ZnRyakFxUEo3UzAifQ
Signer name (self-asserted): proclaim LP-0023
Signer URL (self-asserted): https://github.com/goldennftplatform-svg/proclaim
Signer DID is NOT in trusted keyring
```

- Package: `ui/registry/registry.lgx` — 71 MB, gitignored (`*.lgx`); must be a Release asset, not committed.
- Manifest: `manifestVersion 0.7.0`, `type "ui"`, variant `linux-amd64 -> libregistry_plugin.so`,
  root hash `4979d0f4992d9678cb5b07efc9d12b22938a59bbed0f42f471d5cc477fd53f50`.
- Archive contents: `manifest.json`, `manifest.sig`, `variants/linux-amd64/{Main.qml,libregistry_ffi.so,libregistry_plugin.so}`.
- Signature: Ed25519, `did:jwk`, valid; signer name/URL are self-asserted.

## Honest caveats (do not claim more than this)

1. **Not `logos-module-builder`.** Built with the `lgx` CLI directly. The prize text
   names `mkLogosModule` while the builder's QML helper is `mkLogosQmlModule`
   (`ui_qml`); reviewer interpretation is unresolved. This criterion stays open.
2. **Dev key, not trusted.** The signing key was generated locally and is **not** in
   any consumer's trusted keyring. `lgx verify` proves integrity + self-asserted
   identity, not Logos-wide trust.
3. **Legacy `type: "ui"`.** Newer catalogs/builder templates use `ui_qml` at a newer
   manifest version. Catalog install behaviour in Basecamp was not tested.
4. **Linux-only variant.** Only `linux-amd64` is packaged; no macOS/darwin build,
   so the macOS adoption evidence remains unmet.
5. **Not published / not install-tested.** No GitHub Release, no
   `logos-modules-release-base` catalog, and no install into the Basecamp AppImage
   was performed. That is the next gate.
6. **FFI is Linux `.so`.** `libregistry_ffi.so` is bundled; portable RPATH/loader
   behaviour on other platforms is unverified.

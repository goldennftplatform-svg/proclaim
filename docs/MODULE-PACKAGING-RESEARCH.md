# Logos module packaging / catalog research — LP-0023 `proclaim`

**Date:** 2026-11-19  
**Scope:** research only. No repository code, commits, pushes, releases, or tokens were touched.  
**Method:** public GitHub raw files and GitHub contents/API endpoints, plus read-only inspection of `C:\Users\XPS\Downloads\GFD\stage\Makefile` and local LP-0023 solution/spec files. Unauthenticated GitHub code search was unavailable (`401`), so authoritative details were taken from repository READMEs, docs, templates, workflows, and raw files.

> **Implementation update (2026-11-19):** the manual LGX path in point 3/4 below has now been carried out end-to-end. A signed `registry.lgx` was built locally with the `lgx` CLI (no Nix/Docker): Qt plugin + FFI compiled, `lgx create` → `lgx add` → metadata merge → `lgx keygen`/`lgx sign` → `lgx verify` (structure + signature valid). Two fixes were required: (a) a real C++ bug — `register` is a reserved keyword, renamed to `registerProgram` across header/impl/QML; (b) the manifest patch had to **merge** curated metadata into the `lgx`-generated manifest rather than replace it, or the generated content `hashes` were lost and verification failed with `Missing content hashes in manifest`. The full reproducible commands, verification output and honest caveats are in [`LGX-BUILD.md`](LGX-BUILD.md). This resolves research item 8 in §8 (`lgx` built from source; CMake path works without prebuilt assets) and the manual-path part of item 3 (manifest patch verified by `lgx verify`), but the **release-action sidecar/index checks remain untested** and the module is still **not** builder-generated, catalog-published, or install-tested — those remain pending.

---

## 1. Bottom line

1. `proclaim` currently has a **Qt/QML Basecamp scaffold** at `ui/registry/` and manual `make lgx` / `make lgx-sign` targets. It does **not** yet have a `logos-module-builder` flake, `metadata.json`, `mkLogosModule`/`mkLogosQmlModule` usage, a `logos-modules-release-base` catalog, `logos-repo.json`, `index.json`, or GitHub release assets.
2. The authoritative builder API distinguishes:
   - `mkLogosModule` — core C++ modules and **legacy `type: "ui"`** widget modules.
   - `mkLogosQmlModule` — current **`type: "ui_qml"`** QML views, with an optional C++ backend.
   The prize wording in `SOLUTION-LP-0023.md` says `mkLogosModule`, but the current documented Qt/QML path uses `mkLogosQmlModule`; this mismatch must be confirmed with the reviewer before implementation.
3. `make lgx` produces a single-variant `linux-amd64` `.lgx` manually from the scaffold, patches its manifest, verifies it, and stops. `make lgx-sign` signs that local `.lgx` with a locally generated `devkey` and verifies it again. Neither target publishes anything to a catalog.
4. Without Nix installed locally, the **manual LGX path is feasible** (Qt6 + CMake + Rust + Python + `lgx` CLI), but the **builder path is not**: `mkLogosModule`/`mkLogosQmlModule` are Nix flake functions, and the release action builds `.#lgx-portable` on GitHub runners with Nix.
5. The cleanest catalog publication route is: a builder-based module repository → fork `logos-modules-release-base` → add the module as a submodule with `scripts/add-module.sh` → run the release workflow → publish the resulting `logos-repo.json` raw URL. Difficulty is **medium-high**, mainly because the current scaffold depends on a Rust FFI `.so` built outside Nix.

---

## 2. Authoritative sources consulted

| Source | URL(s) | What it establishes |
|---|---|---|
| `logos-module-builder` README | `https://github.com/logos-co/logos-module-builder`, `https://raw.githubusercontent.com/logos-co/logos-module-builder/master/README.md` | `mkLogosModule` vs `mkLogosQmlModule`, template names, `nix build .#lgx`/`.#lgx-portable`, `nix run .`, UI dev loop. |
| Builder Nix API | `https://raw.githubusercontent.com/logos-co/logos-module-builder/master/docs/nix-api.md` | Function signatures, outputs, `lgx-portable`, external-library inputs. |
| Builder configuration reference | `https://raw.githubusercontent.com/logos-co/logos-module-builder/master/docs/configuration.md` | `type`, `interface`, `main`, `view`, `icon`, `include`, `platforms`, `nix.external_libraries`. |
| Builder quick reference | `https://raw.githubusercontent.com/logos-co/logos-module-builder/master/docs/quick-reference.md` | Minimal `metadata.json`, `flake.nix`, `CMakeLists.txt`, commands. |
| Builder skill: UI module with C++ backend | `https://raw.githubusercontent.com/logos-co/logos-module-builder/master/skills/create-ui-module.md` | `ui-qml-backend` scaffold command and file layout. |
| Builder skill: QML-only module | `https://raw.githubusercontent.com/logos-co/logos-module-builder/master/skills/create-qml-module.md` | `ui-qml` scaffold command and QML-only caveats. |
| Builder templates | `https://api.github.com/repos/logos-co/logos-module-builder/contents/templates`, `https://raw.githubusercontent.com/logos-co/logos-module-builder/master/templates/ui-qml-backend/flake.nix`, `https://raw.githubusercontent.com/logos-co/logos-module-builder/master/templates/ui-qml-backend/metadata.json`, `https://raw.githubusercontent.com/logos-co/logos-module-builder/master/templates/ui-qml-backend/CMakeLists.txt` | Exact current `ui-qml-backend` template content. |
| `logos-package` / LGX | `https://github.com/logos-co/logos-package`, `https://raw.githubusercontent.com/logos-co/logos-package/master/README.md`, `https://api.github.com/repos/logos-co/logos-package/releases?per_page=5` | `lgx create/add/verify/manifest/sign/keygen/keyring/merge/extract` commands, `.lgx` layout, manifest schema, build prerequisites. |
| `logos-modules-release-base` | `https://github.com/logos-co/logos-modules-release-base`, `https://raw.githubusercontent.com/logos-co/logos-modules-release-base/main/README.md`, `https://raw.githubusercontent.com/logos-co/logos-modules-release-base/main/logos-repo.json`, `https://raw.githubusercontent.com/logos-co/logos-modules-release-base/main/.gitmodules`, `https://raw.githubusercontent.com/logos-co/logos-modules-release-base/main/scripts/add-module.sh`, `https://raw.githubusercontent.com/logos-co/logos-modules-release-base/main/.github/workflows/_release-module.yml` | Catalog fork workflow, `logos-repo.json` shape, submodule discovery, signing modes, variant list. |
| `logos-modules-release-action` | `https://github.com/logos-co/logos-modules-release-action`, `https://raw.githubusercontent.com/logos-co/logos-modules-release-action/master/README.md` | Release pipeline reads `metadata.json`, builds `.#lgx-portable`, merges variants, verifies/signs, creates `<module>-v<version>` releases and sidecars, rebuilds `index.json`. |
| `logos-modules-release-tool` | `https://github.com/logos-co/logos-modules-release-tool`, `https://raw.githubusercontent.com/logos-co/logos-modules-release-tool/main/README.md`, `https://raw.githubusercontent.com/logos-co/logos-modules-release-tool/main/docs/catalog-format.md` | Exact catalog format, `logos-repo.json` field rules, `index.json` structure, `index.py` commands. |
| Example catalogs | `https://raw.githubusercontent.com/edenbd1/logos-forum-catalog/main/logos-repo.json`, `https://raw.githubusercontent.com/Tranquil-Flow/logos-forum-catalog/main/logos-repo.json`, `https://raw.githubusercontent.com/Tranquil-Flow/logos-forum-catalog/main/.gitmodules` | Concrete `logos-repo.json` values and submodule entry shape. |
| Target repo | `https://github.com/goldennftplatform-svg/proclaim`, `https://raw.githubusercontent.com/goldennftplatform-svg/proclaim/master/ui/registry/manifest.json`, `https://raw.githubusercontent.com/goldennftplatform-svg/proclaim/master/ui/registry/CMakeLists.txt`, `https://raw.githubusercontent.com/goldennftplatform-svg/proclaim/master/scripts/patch_lgx_manifest.py` | Current scaffold layout, legacy manifest, CMake dependencies, manifest patch behaviour. |
| Local files | `C:\Users\XPS\Downloads\GFD\stage\Makefile`, `C:\Users\XPS\Downloads\GFD\SOLUTION-LP-0023.md`, `C:\Users\XPS\Downloads\GFD\logos-basecamp\docs\spec.md` | Exact local `lgx`/`lgx-sign` commands, unmet prize criteria, Core Module vs UI App lifecycle. |

---

## 3. What the authoritative docs say

### 3.1 `logos-module-builder`, `mkLogosModule`, and `mkLogosQmlModule`

The builder README defines two public functions:

- `logos-module-builder.lib.mkLogosModule` — “core + legacy UI widgets”.
- `logos-module-builder.lib.mkLogosQmlModule` — `ui_qml` QML views, with optional C++ backend.

The configuration reference says the supported `metadata.json` types are:

```text
"core"    — backend/logic module, no UI (use mkLogosModule)
"ui"      — legacy C++ UI widget module (use mkLogosModule)
"ui_qml"  — QML view module with optional C++ backend (use mkLogosQmlModule)
```

It also says a core module that ships a plugin must declare `interface: "universal"` or `"cdylib"`; the older hand-written interface/plugin path is no longer generated for core modules. Legacy `type: "ui"` widget modules are explicitly described as unaffected.

The builder provides these package outputs automatically:

```bash
nix build              # combined library + includes
nix build .#lib
nix build .#generate   # run generators and emit ready-to-build source
nix build .#lgx        # build .lgx package
nix build .#lgx-portable
nix build .#install
nix build .#install-portable
```

For UI modules:

```bash
nix run .              # launch module in Basecamp standalone mode
nix build .#ui-dev
./result/bin/run-logos-standalone-ui
nix run .#ui-tests
```

The release action specifically evaluates `.#lgx-portable` per variant, so a module repository intended for catalog publication must expose that flake output.

### 3.2 `.lgx` format and `lgx` CLI

From `logos-package/README.md`:

- `.lgx` is a `tar.gz` archive:

```text
mymodule.lgx
├── manifest.json
├── manifest.sig
├── assets/
│   └── lidl/
├── variants/
│   ├── linux-amd64/
│   ├── darwin-arm64/
│   └── web/
├── docs/
└── licenses/
```

- Current `manifestVersion` is `0.3.0`; tooling also reads `0.2.x`.
- Manifest `type` values include `"core"`, `"ui"`, `"ui_qml"`, and `"library"`.
- For `type == "ui_qml"`, `view` is the required QML entry point and `main` is optional backend metadata.
- Exact documented commands include:

```bash
lgx create mymodule
lgx add mymodule.lgx --variant linux-amd64 --files ./build/libfoo.so
lgx add mymodule.lgx --variant web --files ./dist --main index.js
lgx add mymodule.lgx --variant linux-amd64 --files ./build/libfoo.so \
  --assets ./assets
lgx add mymodule.lgx --variant darwin-arm64 --files ./dist --view qml/Main.qml
lgx add mymodule.lgx -v linux-amd64 -f ./new-build/libfoo.so -y
lgx verify mymodule.lgx
lgx manifest mymodule.lgx
lgx manifest mymodule.lgx --json > manifest.json
lgx signature mymodule.lgx
lgx keygen --name my-key
lgx sign mymodule.lgx --key my-key
lgx keyring add publisher-name did:jwk:eyJjcnYi... --display-name "Publisher" --url "https://..."
lgx merge linux.lgx darwin.lgx -o mymodule.lgx
lgx extract mymodule.lgx --variant linux-amd64 --output ./extracted
tar -tzf mymodule.lgx
```

`lgx verify` checks content hashes; if signed, it also checks the Ed25519 signature and reports whether the signer DID is in the trusted keyring. Exit code is `0` on success.

Building `lgx` without Nix is documented:

```bash
mkdir build
cd build
cmake ..
make -j$(nproc)
```

Prerequisites listed there are CMake 3.16+, a C++17 compiler, zlib, and ICU. The release-tool README mentions prebuilt releases at `https://github.com/logos-co/logos-package/releases`, but the latest release returned by the API (`0.1.0`) currently has **no assets**; this needs checking before relying on a download.

### 3.3 Catalog files and publication

`logos-repo.json` is the catalog identity card. Required fields are `name`, `displayName`, and `indexUrl`; `schemaVersion: 1` is advisory but recommended. From the release-base template:

```json
{
  "schemaVersion": 1,
  "name": "CHANGE-ME-modules",
  "displayName": "CHANGE ME — My Logos Modules",
  "description": "A short human description of this module catalog.",
  "homepage": "https://example.com/modules",
  "indexUrl": "https://github.com/CHANGE-ME-OWNER/CHANGE-ME-REPO/releases/download/index/index.json",
  "trustedSigners": []
}
```

`index.json` is generated, never hand-edited. Its current schema version is `2`; it lists every package, version, download URL, `rootHash`, embedded manifest, and optional signature. A client walks:

```text
logos-repo.json URL → indexUrl → index.json → .lgx URL → verify against index entry
```

The template layout is:

```text
.
├── logos-repo.json
├── includes.json                         # optional
├── .gitmodules
├── submodules/                           # one git submodule per module
├── scripts/
│   ├── add-module.sh
│   └── catalog.sh
└── .github/workflows/
    ├── _release-module.yml
    ├── release-module.yml.template
    ├── release-all.yml
    ├── rebuild-index.yml
    ├── validate-repo.yml
    └── unpublish.yml
```

Exact publication flow documented by the template:

```bash
git clone https://github.com/<your-owner>/<your-repo>
cd <your-repo>
./scripts/add-module.sh https://github.com/<you>/<your-module-repo>
git add -A && git commit -m "Add <your-module-repo>" && git push
```

Then run **Release all modules** from the Actions tab, or:

```bash
./scripts/catalog.sh release-all
```

The action:

1. Reads `metadata.json` for `name` and `version`.
2. Builds the module `.lgx` on a per-variant matrix.
3. Merges per-variant artifacts with `lgx merge`.
4. Runs `lgx verify`.
5. Optionally signs.
6. Extracts `lgx manifest --json` and `lgx signature` into a sidecar.
7. Creates a GitHub release tagged `<module>-v<version>` with `.lgx` and `sidecar.json`.
8. Dispatches `rebuild-index`, which produces the rolling `index` release.

Default action variants are `darwin-arm64`, `linux-amd64`, and `linux-arm64`; `windows-x86_64` is opt-in. The release-base template currently sets:

```yaml
variants: "darwin-arm64,linux-amd64,linux-arm64,windows-x86_64"
```

and `signing_mode: none`.

Useful catalog validation commands from `logos-modules-release-tool`:

```bash
./index.py validate-repo logos-repo.json --self-url https://example.com/logos-repo.json
./index.py validate index.json
./index.py validate index.json --full
```

`validate --full` requires `lgx` on `PATH`; `validate-repo` is pure JSON validation.

---

## 4. Exact commands and minimal file layout for a Qt/QML Basecamp scaffold

### 4.1 Current canonical `ui_qml` + C++ backend scaffold

This is the documented current path for a QML view with a C++ backend. It uses `mkLogosQmlModule`, not `mkLogosModule`.

```bash
mkdir logos-{module_name}-module && cd logos-{module_name}-module
nix flake init -t github:logos-co/logos-module-builder#ui-qml-backend
git init && git add -A
```

Minimal layout from the skill and template:

```text
logos-{module_name}-module/
├── flake.nix
├── metadata.json
├── CMakeLists.txt
└── src/
    ├── {module_name}.rep              # QtRO view contract
    ├── {module_name}_backend.h        # *Backend class you write
    ├── {module_name}_backend.cpp
    ├── qml/
    │   └── Main.qml
    └── icons/
        └── icon.png                   # 256x256 PNG required for ui_qml at manifest 0.4.0+
```

`flake.nix` from the template:

```nix
{
  description = "Logos ui_qml module (C++ backend + QML view) — replace with your description";

  inputs = {
    logos-module-builder.url = "github:logos-co/logos-module-builder";
  };

  outputs = inputs@{ logos-module-builder, ... }:
    logos-module-builder.lib.mkLogosQmlModule {
      src = ./.;
      configFile = ./metadata.json;
      flakeInputs = inputs;
    };
}
```

Representative `metadata.json` from the template (renamed placeholders shown as-is from the template):

```json
{
  "name": "ui_example",
  "display_name": "UI Example",
  "version": "1.0.0",
  "type": "ui_qml",
  "interface": "universal",
  "category": "example",
  "description": "A C++ UI module: QML view + a backend that runs as a core module",
  "main": "ui_example_plugin",
  "backend": "module",
  "icon": "src/icons/icon.png",
  "view": "qml/Main.qml",
  "dependencies": [],
  "provides": [],
  "uses": [],
  "codegen": { "rep": "src/ui_example.rep" },

  "nix": {
    "packages": { "build": [], "runtime": [] },
    "external_libraries": [],
    "cmake": {
      "find_packages": [],
      "extra_sources": [],
      "extra_include_dirs": [],
      "extra_link_libraries": []
    }
  }
}
```

Build/test commands:

```bash
nix build
nix build .#lgx
nix build .#lgx-portable
nix run .
nix build .#ui-dev
./result/bin/run-logos-standalone-ui
nix run .#ui-tests
nix develop
```

### 4.2 Literal `mkLogosModule` scaffold

If the prize reviewer requires the literal function name, the documented `mkLogosModule` route is either a core module or the legacy `type: "ui"` widget path. There is no current builder template for “Qt/QML view via `mkLogosModule`”; current QML views use `mkLogosQmlModule`.

Minimal core module scaffold:

```bash
mkdir logos-my-module && cd logos-my-module
nix flake init -t github:logos-co/logos-module-builder
git init && git add -A
```

Layout:

```text
logos-my-module/
├── metadata.json
├── flake.nix
├── CMakeLists.txt
└── src/
    ├── my_module_impl.h
    └── my_module_impl.cpp
```

`flake.nix` from the template:

```nix
{
  inputs = {
    logos-module-builder.url = "github:logos-co/logos-module-builder";
  };

  outputs = inputs@{ logos-module-builder, ... }:
    logos-module-builder.lib.mkLogosModule {
      src = ./.;
      configFile = ./metadata.json;
      flakeInputs = inputs;
    };
}
```

Basic `metadata.json` from the quick reference:

```json
{
  "name": "my_module",
  "display_name": "My Module",
  "version": "1.0.0",
  "type": "core",
  "interface": "universal",
  "category": "general",
  "description": "My module",
  "main": "my_module_plugin",
  "dependencies": [],
  "nix": {
    "packages": { "build": [], "runtime": [] },
    "external_libraries": [],
    "cmake": { "find_packages": [], "extra_sources": [] }
  }
}
```

For a legacy `ui` widget module, the configuration reference says `type: "ui"` uses `mkLogosModule`, but the current template set does not include such a UI template. Reusing the existing generated `RegistryBackend.cpp` / `RegistryPlugin.cpp` under that path is therefore **unverified** and should be treated as a fallback only after reviewer confirmation.

### 4.3 QML-only variant

If the C++ backend is removed and all logic lives in QML/JS:

```bash
mkdir logos-{name}-module && cd logos-{name}-module
nix flake init -t github:logos-co/logos-module-builder#ui-qml
```

Layout:

```text
logos-{name}-module/
├── flake.nix
├── metadata.json
└── Main.qml
```

The skill warns that `logos.callModuleAsync()` / `logos.callModule()` from QML is deprecated because QML will lose core access in a future breaking release; logic should move to a backend. That makes this route a poor fit for the registry, which already has a C++ backend and Rust FFI.

---

## 5. What local `make lgx` and `make lgx-sign` actually do

Source: `C:\Users\XPS\Downloads\GFD\stage\Makefile`, matching the target repo's `master` Makefile.

Relevant variables:

```make
UI_OUT_DIR := ui/registry
FFI_LIB := target/debug/libregistry_ffi.so
LGX_FILE := $(UI_OUT_DIR)/registry.lgx
LGX_STAGING := $(UI_OUT_DIR)/.lgx-staging
ARCH := $(shell uname -m | sed 's/x86_64/amd64/;s/aarch64/arm64/')
VARIANT := linux-$(ARCH)
```

`make lgx` depends on `ui-build`, then performs exactly:

1. Checks `lgx` is on `PATH`; error message points to `https://github.com/logos-co/logos-package`.
2. Checks `target/debug/libregistry_ffi.so` exists.
3. Removes `ui/registry/.lgx-staging` and `ui/registry/registry.lgx`.
4. Creates `ui/registry/.lgx-staging`.
5. Copies into staging:
   - `ui/registry/build/libregistry_plugin.so`
   - `target/debug/libregistry_ffi.so`
   - `ui/registry/qml/Main.qml`
6. Runs:

```bash
cd ui/registry && lgx create registry
lgx add ui/registry/registry.lgx -v linux-amd64 -f ui/registry/.lgx-staging -m libregistry_plugin.so -y
python3 scripts/patch_lgx_manifest.py ui/registry/registry.lgx ui/registry/manifest.json
lgx verify ui/registry/registry.lgx
```

7. Removes the staging directory.

`lgx add` replaces the whole variant if it already exists; `-y` skips confirmation. `-m libregistry_plugin.so` marks the plugin entry point. The resulting archive contains only the `linux-amd64` variant (or `linux-arm64` on ARM hosts), with the plugin, FFI `.so`, and QML file at the variant root.

`scripts/patch_lgx_manifest.py` reads the tar.gz LGX, replaces `manifest.json` with `ui/registry/manifest.json`, filters `manifest.main` to variant directories that actually exist, and writes the archive back deterministically (`mtime=0`).

The patched manifest currently in `proclaim` is legacy-shaped:

```json
{
  "author": "",
  "category": "tools",
  "dependencies": [],
  "description": "Qt/QML Basecamp module for the registry program",
  "icon": "",
  "main": {
    "linux-amd64": "libregistry_plugin.so",
    "linux-amd64-dev": "libregistry_plugin.so",
    "linux-x86_64-dev": "libregistry_plugin.so"
  },
  "manifestVersion": "0.2.0",
  "name": "registry",
  "type": "ui",
  "version": "0.1.0"
}
```

`make lgx-sign` performs exactly:

```bash
lgx sign ui/registry/registry.lgx --key devkey
lgx verify ui/registry/registry.lgx
```

It first checks the LGX file and `lgx` binary exist. The Makefile help text documents one-time key creation as:

```bash
lgx keygen --name devkey
```

Per the LGX README, signing validates the package, writes `manifest.sig` with a `did:jwk:...` Ed25519 identity, and `lgx verify` then checks the signature and keyring trust status.

Neither target creates a catalog, `index.json`, GitHub release, or sidecar. The help text says to share the file and install via Basecamp “Install Plugin”; that is direct file distribution, not catalog publication.

---

## 6. Prerequisites without Nix / Docker

### 6.1 Already possible locally (manual LGX path)

No Nix, Docker, `logos-module-builder`, or `lgs` is needed for the `make lgx` / `make lgx-sign` chain itself:

- `lgx` CLI on `PATH`.
  - Build from `logos-package` with CMake: CMake 3.16+, C++17 compiler, zlib, ICU.
  - Or verify a prebuilt release asset exists; the current `0.1.0` release has no assets, so do not assume this.
- Python 3 for `scripts/patch_lgx_manifest.py`.
- Qt6 CMake components used by `ui/registry/CMakeLists.txt`: `Core`, `Gui`, `Widgets`, `Qml`, `Quick`, `QuickWidgets`, `Concurrent`.
- CMake (the scaffold uses `cmake_minimum_required(VERSION 3.16)`).
- Rust/cargo to build `target/debug/libregistry_ffi.so` via `cargo build -p registry_ffi`.
- If regenerating the scaffold/FFI, `spel-client-gen` at the hardcoded default path or overridden via `make SPEL_CLIENT_GEN=...`.
- For signing, a one-time `lgx keygen --name devkey`.
- Basecamp AppImage is only needed for runtime smoke testing, not for building or signing the LGX.

Docker is not invoked by `make lgx`, `make lgx-sign`, `make ui-build`, `make ffi`, or `make idl`. The project README notes that `make build` (guest binary via `cargo risczero`) may require Docker and documents a local-toolchain alternative; that is unrelated to packaging.

### 6.2 Not possible locally without Nix

The builder and catalog action are Nix-based:

- `nix flake init -t github:logos-co/logos-module-builder#ui-qml-backend`
- `nix build`, `nix build .#lgx`, `nix build .#lgx-portable`
- `nix run .`, `nix run .#ui-tests`

The release action installs Nix on GitHub-hosted runners and reads the public Logos cache (`cache.nix.logos.co`), so a developer can avoid a local Nix install only by doing builder builds in GitHub Actions. There is no documented non-Nix invocation of `mkLogosModule` or `mkLogosQmlModule`.

---

## 7. Proposed plan and difficulty

| Phase | Action | Difficulty | Notes |
|---|---|---:|---|
| 0 | Confirm with the prize reviewer whether `mkLogosQmlModule` satisfies “built with `logos-module-builder` (`mkLogosModule`)”. | Low, blocking | Current docs say UI/QML uses `mkLogosQmlModule`; `mkLogosModule` is for core/legacy `ui`. |
| 1 | Create a dedicated module repository (e.g. `goldennftplatform-svg/logos-registry-module`) from `nix flake init -t github:logos-co/logos-module-builder#ui-qml-backend`; commit, enable Nix flakes. | Medium | Requires local Nix or GitHub Actions with Nix. |
| 2 | Port `ui/registry/qml/Main.qml` to `src/qml/Main.qml`; add a 256×256 icon; replace the generated `RegistryBackend` API with a `.rep` contract plus a `*Backend` class deriving the generated source and `LogosModuleContext`. | Medium-high | The current QML file is ~193 KB and was generated against a legacy `type: "ui"` plugin; API compatibility must be tested. |
| 3 | Solve `libregistry_ffi.so` packaging. Options: build the Rust FFI in Nix as an external library input; or vendor the prebuilt `.so` via `nix.external_libraries[].vendor_path` and `include`/`platforms`; or move registry calls into a proper `mkLogosModule` core backend and have the QML backend call it. | High | This is the main technical risk. Vendoring is simplest but likely Linux-only; Nix Rust build is cleaner but needs verifying against the project's Cargo dependencies. |
| 4 | Add `tests/ui-tests.mjs` with a basic load test; run `nix run .#ui-tests`; smoke-test `nix run .` in Basecamp. | Medium | Requires Basecamp that supports `ui_qml` backends; see builder README. |
| 5 | Verify `nix build .#lgx-portable`; inspect with `lgx manifest <pkg>` and `lgx verify <pkg>`. | Medium | This is the artifact the catalog action consumes. |
| 6 | Fork `logos-modules-release-base`; edit `logos-repo.json` (`name`, `displayName`, `homepage`, `indexUrl`); add the module with `./scripts/add-module.sh https://github.com/<owner>/<repo>`; commit/push. | Low-medium | Keep `signing_mode: none` initially; adjust `variants` in `_release-module.yml` if only `linux-amd64` is supported. |
| 7 | Run “Release all modules” or `./scripts/catalog.sh release-all`; confirm `<module>-v<version>` release has `.lgx` + `sidecar.json`, and that the `index` release has regenerated `index.json`. | Medium | Requires Actions permissions `contents: write`. |
| 8 | Validate with `./index.py validate-repo logos-repo.json --self-url <raw-url>` and `./index.py validate index.json --full` if publishing outside the template flow. | Low-medium | `validate --full` needs `lgx` on PATH. |
| 9 | Update `SOLUTION-LP-0023.md` with the `logos-repo.json` raw URL and mark the two pending module criteria complete only after Basecamp install succeeds. | Low | Do not mark complete based only on release creation. |

**Overall difficulty: medium-high.** If the existing generated C++ backend can be vendored unchanged under a builder flake, the path is mostly plumbing. If it must be rewritten into the universal `ui_qml` `.rep` model and the Rust FFI must be built hermetically in Nix, it becomes a substantial port.

**Recommended order:** confirm reviewer wording first; then build a minimal `ui-qml-backend` module with no FFI to prove the Nix/Actions/catalog pipeline; then port the real registry UI and FFI. This isolates catalog failures from porting failures.

---

## 8. Unverified / needs checking in Basecamp

These items were not verifiable from public docs and must be checked before claiming completion:

1. **Reviewer interpretation:** whether `mkLogosQmlModule` counts as satisfying the literal prize wording “built with `logos-module-builder` (`mkLogosModule`)”. Public docs clearly separate the two functions.
2. **Legacy `type: "ui"` in a catalog:** the local manifest uses `type: "ui"` and `manifestVersion: 0.2.0`, while newer catalog examples and builder templates use `ui_qml` / newer manifests. `lgx` reads both, but current catalog install behaviour in Basecamp was not verified.
3. **Manifest patching with release action:** the builder produces its own LGX manifest; the local `patch_lgx_manifest.py` workflow is separate. Whether a patched `type: "ui"` LGX would pass the release action’s sidecar/index checks was not tested.
4. **FFI packaging:** whether `nix.external_libraries[].vendor_path` can stage `libregistry_ffi.so` into a portable `.lgx` with the correct RPATH/loader behaviour is unverified. A vendored `.so` is almost certainly Linux-specific.
5. **Cross-platform variants:** current `make lgx` emits only `linux-amd64`; the release template lists four variants. Either scope the catalog to `linux-amd64` or implement Darwin/ARM/Windows builds. The action skips variants whose flake does not expose `lgx-portable`, but fails setup if none are exposed.
6. **QML runtime compatibility:** `Main.qml` may rely on legacy `logos.callModule(...)`; current docs mark that path deprecated and prefer a backend. The UI must be loaded in the actual Basecamp AppImage after catalog install.
7. **Icon requirement:** `ui_qml` at manifest version 0.4.0+ requires a 256×256 PNG icon; none is present in the current `ui/registry/` scaffold.
8. **`lgx` availability:** docs mention prebuilt releases, but the latest `logos-package` release has no assets. Confirm before avoiding a CMake build of `lgx`.
9. **`indexUrl` style:** both `https://raw.githubusercontent.com/.../main/index.json` (edenbd1 example) and `https://github.com/.../releases/download/index/index.json` (template, Tranquil-Flow example) appear in public catalogs. Follow the template unless a client test proves otherwise.
10. **No live install test performed:** this research did not install a catalog package into Basecamp, sign with a production key, or push any GitHub release. Completion requires an end-to-end install from the published `logos-repo.json` URL.

---

## 9. Concrete next commands once prerequisites are available

These are quoted from the authoritative docs, not invented:

```bash
# builder module
mkdir logos-registry-module && cd logos-registry-module
nix flake init -t github:logos-co/logos-module-builder#ui-qml-backend
git init && git add -A
nix build .#lgx-portable
lgx manifest ./result/lgx/*.lgx 2>/dev/null || true
lgx verify <built-lgx-path>
nix run .

# catalog repo (fork of logos-modules-release-base)
./scripts/add-module.sh https://github.com/<owner>/logos-registry-module
git add -A && git commit -m "Add logos-registry-module" && git push
./scripts/catalog.sh release-all

# validation from a clone of logos-modules-release-tool
./index.py validate-repo logos-repo.json --self-url <raw-logos-repo-json-url>
./index.py validate index.json --full
```

The `lgx manifest ./result/lgx/*.lgx` line is illustrative of where a built artifact may appear; the authoritative docs do not state that exact result path. Use `nix build .#lgx-portable` and inspect `./result/` after the build rather than assuming a filename.

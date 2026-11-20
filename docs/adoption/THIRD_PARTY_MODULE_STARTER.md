<!-- SPDX-License-Identifier: MIT AND Apache-2.0 -->

# Third-party Logos UI module starter (LP-0023 registry)

This guide is for an independent developer who wants their Logos UI module to
**read** the LEZ Program Registry. It describes the localnet integration surface
that exists today. The project is an **early draft**: the read-only API and SDKs
work on localnet, **testnet 0.4 is pending**, and the Qt/QML module in
`ui/registry/` is **only a scaffold** — an installable `.lgx` package is
**pending**. There is no adoption to assume here; you would be an independent
integrator.

Repo: <https://github.com/goldennftplatform-svg/proclaim> (MIT AND Apache-2.0)

## 0. Where to get the API and SDKs

| Surface | Source | Use it when |
| --- | --- | --- |
| Read-only HTTP API | `registry-api/` in the repo; runs on `127.0.0.1:3030` | Any language; simplest first integration |
| JS/TS client | `clients/js` → `@lp0023/registry-client` | ESM JS/TS, Node 18+ or browser `fetch` |
| Rust SDK | `registry_core` crate in the repo | Rust hosts, shared Borsh types, PDA helpers, filters |
| C FFI | generated header `registry_ffi/generated/registry.h` + `registry_client.rs` | C/C++/Qt/QML module backends |

Start the API locally (it never submits transactions):

```bash
source ~/.cargo/env
export RUSTFLAGS='--cfg getrandom_backend="custom"'
export LEE_WALLET_HOME_DIR="$PWD/.scaffold/wallet"   # wallet with the sequencer connection
make api                       # or: cargo run -p registry-api --bin registry-api
curl -s http://127.0.0.1:3030/health   # {"status":"ok"}
```

The full walkthrough is
[`docs/INTEGRATION.md`](https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/INTEGRATION.md).

## 1. Exact HTTP endpoints

All endpoints are `GET`. Success responses are JSON; errors are JSON
`{"error":"..."}` (invalid query params → `400`, missing entry → `404`,
upstream failures → `502`/`500`).

| Endpoint | Returns |
| --- | --- |
| `/health` | `{"status":"ok"}` |
| `/index` | Array of registered program IDs (base58 strings) |
| `/entries` | Array of full decoded `RegistryEntry` objects |
| `/search?name=…&tag=…&author=…&verified=…&limit=…` | Filtered entries |
| `/entry/:program_id` | Single entry by base58 program ID; `404` if not indexed |

Filter semantics for `/search`:
- `name` — case-insensitive **substring** match.
- `tag` — **repeatable**, OR-matched, case-insensitive substring match.
- `author` — optional; base58/hex/byte-array account.
- `verified` — optional boolean; `true` returns only independently verified entries.
- `limit` — optional maximum number of entries.
- Different filter kinds compose with AND; tags with each other with OR.

```bash
curl -s http://127.0.0.1:3030/health
curl -s http://127.0.0.1:3030/index
curl -s http://127.0.0.1:3030/entries

# Repeat tag for OR filtering; use --get --data-urlencode so CIDs/special chars are escaped.
curl -sG http://127.0.0.1:3030/search \
  --data-urlencode 'name=registry' \
  --data-urlencode 'tag=spel' \
  --data-urlencode 'tag=lez' \
  --data-urlencode 'verified=true' \
  --data-urlencode 'limit=10'

# Single entry by program ID:
curl -s http://127.0.0.1:3030/entry/6FMaVQ6CmbAVNHgwsmmeBe5kiiKkQRFtqu4GUQ2vAf2T
```

Note: on the wire, `program_id`, `author` and `verifier` may appear as JSON
byte arrays (`[u8; 32]`) or base58 strings depending on server version. The JS
client normalizes both to base58 for you.

## 2. Exact JS client snippet

Dependency-free ESM; install from the repo checkout with `npm install ./clients/js`
(or depend on `@lp0023/registry-client` where published). Uses global `fetch`
(Node 18+). It builds queries with `URLSearchParams`, so escaping is handled.

```js
import { RegistryClient, RegistryApiError, idlAvailability } from '@lp0023/registry-client';

const client = new RegistryClient('http://127.0.0.1:3030');

try {
  // name: case-insensitive substring; tags: repeatable OR; verified/limit optional.
  const entries = await client.search({
    name: 'registry',
    tags: ['spel', 'lez'],
    verified: true,
    limit: 20,
  });

  for (const e of entries) {
    // Render metadata regardless of IDL state; degrade gracefully below.
    const idlLabel = idlAvailability(e) === 'available' ? e.idl_cid : '(no IDL)';
    console.log(e.program_id, e.name, e.version, idlLabel,
      e.source_verified ? 'verified' : 'unverified');
  }

  // One entry by program ID (base58, hex, or byte array accepted):
  const one = await client.entry('6FMaVQ6CmbAVNHgwsmmeBe5kiiKkQRFtqu4GUQ2vAf2T');
  console.log(one.description, one.code_cid, one.commit, one.verifier);
} catch (err) {
  if (err instanceof RegistryApiError) {
    console.error(`API ${err.status}:`, err.body);   // status 0 = fetch failed
    process.exitCode = 1;
  } else {
    throw err;
  }
}
```

Other client methods: `client.health()`, `client.index()`, `client.entries()`.

## 3. Expected `RegistryEntry` fields

Keys are normalized to base58 strings by the client.

| Field | Type | Meaning |
| --- | --- | --- |
| `program_id` | string (base58) | 32-byte program ID; also the entry PDA seed |
| `name` | string | Human-readable name |
| `version` | string | Version string |
| `author` | string (base58) | Account that registered / may update the entry |
| `description` | string | One-line purpose |
| `tags` | string | Comma-separated keywords (API matches OR) |
| `idl_cid` | string | Logos Storage CID of the IDL document |
| `code_cid` | string | Logos Storage CID of the source archive |
| `commit` | string | Git commit that produced the deployed ELF |
| `build_config` | string | Settings needed to reproduce the ELF |
| `timestamp` | number | Registration time, Unix seconds |
| `deployer_claimed` | boolean | Signer claimed to be the deployer (**not** independent proof) |
| `source_verified` | boolean | An independent rebuild matched the deployed bytecode |
| `verifier` | string \| null | Base58 verifier account, or `null` while unverified |
| `verification_method` | string | e.g. `reproducible-rebuild:program-id` |
| `evidence_cid` | string | CID of the rebuild log/attestation |

`deployer_claimed` is a self-assertion. Treat an entry as independently verified
only when **`source_verified` is true and `verifier` is non-empty** — that is
exactly what the guest's `is_independently_verified()` requires.

## 4. Deriving the entry PDA / looking up one entry

You do **not** need a central index to find a single program:

- **Entry PDA**: seed is the 32-byte program ID itself (`pda = arg("key")` in the
  guest; `--key <program-id>` on the CLI). Derive it locally, deterministically.
  Example (localnet v5 registry): entry PDA
  `EP644ahCbBehPMV8krDKDnToxJwfaMGaKKa1NzDFcM6q`.
- **Index PDA**: single global list, seed `registry_index` (right-padded to 32
  bytes), e.g. `9mr5hePKo24ZHNztzVpuRND7PeAsR17yQ2M5HHPKcbw3` on localnet.

```bash
# Derive locally, no on-chain lookup needed:
lgs spel -- pda entry --key 4df7d5396b68a16aae6d0f768ef447a0055eb43866827154f71139e486265144
lgs spel -- pda index

# Or skip derivation entirely and ask the read-only API:
curl -s http://127.0.0.1:3030/entry/6FMaVQ6CmbAVNHgwsmmeBe5kiiKkQRFtqu4GUQ2vAf2T
```

For enumeration, walk `/index` → derive each entry PDA → fetch the record, or
let the API/client do it. Localnet v5 registry program:
hex `4df7d5396b68a16aae6d0f768ef447a0055eb43866827154f71139e486265144`,
base58 `6FMaVQ6CmbAVNHgwsmmeBe5kiiKkQRFtqu4GUQ2vAf2T`.

## 5. Graceful degradation for IDLs (`idl_availability` / `idlAvailability`)

Entry metadata lives on-chain, so a missing, placeholder, or unresolvable IDL
must **never** break rendering of the entry itself.

- JS/TS: `idlAvailability(entry)` returns `'available'` or `'unavailable'`.
  Empty, `none`, `null`, or `placeholder` CIDs are `'unavailable'`.
- Rust: `registry_core::RegistryEntry::idl_availability()` returns the same two
  states.

Recommended UI behaviour:

1. Render `name`, `version`, `description`, `tags`, and verification state from
   the entry record unconditionally.
2. Call `idlAvailability(e)` before trying to resolve `idl_cid`. On
   `'unavailable'`, show a neutral "no IDL" label and keep the entry usable —
   do not hide it, throw, or block the list.
3. Only attempt CID resolution on `'available'`; if resolution still fails,
   keep the metadata view and surface a non-fatal notice.
4. CIDs such as `zDv*8iXZ6U` contain `*` — escape them with `URLSearchParams`,
   `encodeURIComponent`, or `curl --get --data-urlencode` when putting them in
   URLs.

## 6. Register your own program and verify it independently

1. **Prepare artifacts**: `make idl` produces the IDL JSON; upload the IDL and a
   source archive of the exact commit to Logos Storage via
   `tools/qtro_storage_upload.py` (the storage module is a QtRO source on a Unix
   socket, **not** HTTP) and capture the CIDs. Quote CIDs in the shell.
2. **Register** (see `docs/INTEGRATION.md` step 3 for the full command): include
   `program_id` as `--key`, `name`, `version`, `description`, `tags`, `idl_cid`,
   `code_cid`, `commit`, `build_config`, `timestamp`, `author`, and
   `deployer_claimed` (only if true — it is a claim, not proof). Leave the
   verification fields empty. Only the original `author` may later `update`.
3. **Verify from a different account**: fetch the exact `code_cid`, rebuild from
   `commit`/`build_config`, wrap the ELF in R0BF, derive the program ID and
   compare. `scripts/verify-source.sh` automates the clean-room rebuild and
   prints a ready-to-submit `mark-verified` command plus an attestation template.
   The guest rejects self-verification: `verifier` must differ from `author`.
4. **Remember**: any source-changing `update` (`code_cid`, `commit`, `idl_cid`,
   or `build_config`) clears the prior attestation. Re-run the rebuild and
   `mark-verified` afterwards.

## 7. UI packaging status (read this before planning an install flow)

The `ui/registry/` directory is a **generated Qt/QML Basecamp scaffold** from
`make ui-gen`; `make ui-regen` regenerates the C++ backend while keeping your
hand-written `qml/Main.qml`. `make lgx` / `make lgx-sign` targets exist, but the
module has **not** yet been built with `logos-module-builder` and published to a
catalog. Treat the scaffold as a starting point for your own module, not an
installable artifact. Testnet 0.4 program IDs are also still pending.

<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# Integrate your LEZ program with the registry in 15 minutes

This tutorial takes a deployed LEZ program from "I have a binary" to "anyone can
find it, inspect its IDL/source, and check an independent rebuild attestation".
Everything below is read-only except the two transactions in steps 3 and 5.
The registry is dual-licensed under MIT OR Apache-2.0.

## Constants you will need

| Item | Value |
| --- | --- |
| Registry program ID (v5, hex) | `4df7d5396b68a16aae6d0f768ef447a0055eb43866827154f71139e486265144` |
| Registry program ID (base58) | `6FMaVQ6CmbAVNHgwsmmeBe5kiiKkQRFtqu4GUQ2vAf2T` |
| Global index PDA | `9mr5hePKo24ZHNztzVpuRND7PeAsR17yQ2M5HHPKcbw3` |
| Example entry PDA | `EP644ahCbBehPMV8krDKDnToxJwfaMGaKKa1NzDFcM6q` |
| Example IDL CID | `zDv*8iXZ6U` |
| Example source CID | `zDv*yXuozL` |
| Read-only JSON API | `http://127.0.0.1:3030` |

The `*` in those CIDs is a normal Logos Storage CID character. It is safe in a
JSON body, but when a CID goes into a URL (query string or path) let
`URLSearchParams`/`encodeURIComponent` or `curl --get --data-urlencode` do the
escaping rather than pasting it into a raw URL.

## 0. Start the read-only API (2 min)

```bash
cd ~/logos-work/registry
source ~/.cargo/env
export RUSTFLAGS='--cfg getrandom_backend="custom"'
export LEE_WALLET_HOME_DIR="$PWD/.scaffold/wallet"
make api                       # or: cargo run -p registry-api --bin registry-api
curl -s http://127.0.0.1:3030/health   # {"status":"ok"}
```

The API never submits transactions; it reads the chain, derives PDAs, decodes
Borsh account data and filters results. `LEE_WALLET_HOME_DIR` must point at the
wallet that holds the sequencer connection.

## 1. Derive your PDAs (3 min)

There are exactly two account types:

- **Entry PDA** — one per program. Its seed is the 32-byte program ID itself;
  in the guest this is expressed as `pda = arg("key")`. On the CLI the flag is
  `--key` (`--program-id` is a deprecated global flag, not the seed).
- **Index PDA** — a single global list of registered program IDs, derived from
  the registry program ID with the fixed seed `registry_index` (right-padded to
  32 bytes). It is created once with `init-index`.

```bash
PID=4df7d5396b68a16aae6d0f768ef447a0055eb43866827154f71139e486265144

lgs spel -- pda entry --key "$PID"   # deterministic; no on-chain lookup needed
lgs spel -- pda index                # 9mr5hePKo24ZHNztzVpuRND7PeAsR17yQ2M5HHPKcbw3
```

`register` appends your program ID to that index. Readers can therefore walk
`/index`, derive each entry PDA, and fetch the record without a central
database.

## 2. Prepare the record fields (4 min)

Put these in the `register` transaction:

| Field | What to put there |
| --- | --- |
| `program_id` | Your 32-byte program ID, passed as `--key`; also the entry PDA seed. The API exposes it as base58. |
| `name`, `version`, `description` | Human-readable identity, e.g. `My LEZ wallet`, `0.1.0`, one-line purpose. |
| `idl_cid` | Logos Storage CID of the generated IDL document (`make idl` → `registry-idl.json`), e.g. `zDv*8iXZ6U`. |
| `code_cid` | Logos Storage CID of the source archive containing the exact commit, e.g. `zDv*yXuozL` (a `.tar.gz`). |
| `commit` | Full git SHA of the commit that produced the deployed ELF (`git rev-parse HEAD`). |
| `build_config` | Everything needed to rebuild the ELF: toolchain and target, e.g. `guest=local,risc0=3.0.5,target=riscv32im-risc0-zkvm-elf`, or a pinned image digest if you publish one. |
| `tags` | Comma-separated keywords, e.g. `registry,spel,lez,wallet`. Tags are matched as case-insensitive substrings with **OR** semantics. |
| `author` | The named signer account (`Public/...`) that registers and later may update the entry. |
| `timestamp` | Unix seconds, `$(date +%s)`. |
| `deployer_claimed` | `true` only if the signer deployed the program; this is a claim, not proof. |

Leave `source_verified`, `verifier`, `verification_method` and `evidence_cid`
empty at registration — an independent verifier fills them in step 5.

Upload the IDL and source archive with the QtRO storage helper (the storage
module is **not** an HTTP service):

```bash
# terminal 1: isolated host with a caller-controlled token (keep running)
tools/launch_storage_host.sh

# terminal 2: upload both artifacts and capture their CIDs
python3 tools/qtro_storage_upload.py \
  --token-file /tmp/proclaim_storage_token \
  --init-json '{"data-dir":"/home/xps/proclaim-storage-data","network":"logos.test"}' \
  --host-log <host-stderr-log> \
  --out /tmp/cids.json \
  artifacts/registry-idl.json artifacts/proclaim-source-v5.tar.gz

cat /tmp/cids.json    # read the IDL and source CIDs for --idl-cid / --code-cid
```

See `tools/qtro_storage_upload.py` for the event/log-based CID capture details.

## 3. Register (2 min)

```bash
cd ~/logos-work/registry
AUTHOR=Public/HQk3STQocrbw16EhAMfSphd6fYrrnSdMHcG1KbfskDdh
PID=4df7d5396b68a16aae6d0f768ef447a0055eb43866827154f71139e486265144
IDL_CID='zDv*8iXZ6U'
CODE_CID='zDv*yXuozL'
COMMIT="$(git rev-parse HEAD)"
TIMESTAMP="$(date +%s)"

# One-time only, if the global index does not exist yet:
lgs spel -- init-index --authority "$AUTHOR"

lgs spel -- register --key "$PID" \
  --name "My LEZ module" --version "0.1.0" \
  --description "Example LEZ program registered with LP-0023" \
  --tags "registry,spel,lez,wallet" \
  --idl-cid "$IDL_CID" --code-cid "$CODE_CID" \
  --commit "$COMMIT" \
  --build-config "guest=local,risc0=3.0.5,target=riscv32im-risc0-zkvm-elf" \
  --timestamp "$TIMESTAMP" \
  --deployer-claimed true --author "$AUTHOR"

lgs spel -- pda entry --key "$PID"          # copy this entry PDA below
lgs spel -- inspect <ENTRY_PDA> --type RegistryEntry
lgs spel -- inspect 9mr5hePKo24ZHNztzVpuRND7PeAsR17yQ2M5HHPKcbw3 --type RegistryIndex
```

Quote CIDs (`'zDv*8iXZ6U'`) so the shell does not glob them. Only the original
`author` may later call `update`.

## 4. Search with the HTTP API or the JS client (2 min)

All endpoints are `GET`; errors are JSON `{"error":"..."}`. `name` is a
case-insensitive substring match; tags repeat and are OR-matched (also
case-insensitive); `author`, `verified` and `limit` are optional. Filter kinds
compose with AND, tags with each other with OR.

```bash
curl -s http://127.0.0.1:3030/health
curl -s http://127.0.0.1:3030/index
curl -s http://127.0.0.1:3030/entries
curl -sG http://127.0.0.1:3030/search \
  --data-urlencode 'name=LP-0023' \
  --data-urlencode 'tag=registry' \
  --data-urlencode 'tag=spel' \
  --data-urlencode 'verified=true' \
  --data-urlencode 'limit=10'
curl -s http://127.0.0.1:3030/entry/6FMaVQ6CmbAVNHgwsmmeBe5kiiKkQRFtqu4GUQ2vAf2T
```

`program_id`, `author` and `verifier` may be JSON byte arrays (`[u8; 32]`)
or base58 strings depending on the server version; the JS client normalises
both to base58.

Copy-paste JS example:

```bash
npm install ./clients/js
cat > search.mjs <<'EOF'
import { RegistryClient, RegistryApiError, idlAvailability } from '@lp0023/registry-client';

const client = new RegistryClient('http://127.0.0.1:3030');

try {
  const entries = await client.search({
    name: 'registry', tags: ['registry', 'spel'], limit: 10,
  });
  for (const e of entries) {
    // Metadata renders even when the IDL CID is missing or unresolvable.
    const idl = idlAvailability(e) === 'available' ? e.idl_cid : '(no IDL)';
    console.log(e.program_id, e.name, e.version, idl,
                e.source_verified ? 'verified' : 'unverified');
  }

  const one = await client.entry('6FMaVQ6CmbAVNHgwsmmeBe5kiiKkQRFtqu4GUQ2vAf2T');
  console.log(one.code_cid, one.commit, one.verifier);
} catch (err) {
  if (err instanceof RegistryApiError) {
    console.error(`API ${err.status}:`, err.body);
    process.exitCode = 1;
  } else {
    throw err;
  }
}
EOF
node search.mjs
```

The client uses the global `fetch` (Node 18+) and has no dependencies. Internally
it builds queries with `URLSearchParams`, so tags and other values are escaped.
See `clients/js/README.md` for the full API surface.

## 5. Independent verification (2 min)

A `deployer_claimed` flag is just the registrant's assertion. Verification is a
separate, attributed action:

1. A **different account** (never the registering `author`) fetches the exact
   `code_cid` archive, rebuilds from `commit`/`build_config`, wraps the ELF in
   R0BF, derives its program ID, and compares it with the on-chain entry.
2. `scripts/verify-source.sh` automates this clean-room rebuild and prints a
   ready-to-submit `mark-verified` command plus an `attestation.json` template.

```bash
PID=4df7d5396b68a16aae6d0f768ef447a0055eb43866827154f71139e486265144
EVIDENCE_CID='zDv*8iXZ6U'       # CID of the rebuild log/attestation
scripts/verify-source.sh "$PID" <SOURCE_DIR> "$EVIDENCE_CID"

VERIFIER=Public/CpSH8cuTUunA18SUEZ8Ky4SGbjQYW3ndJUbCLsigtuCN
lgs spel -- mark-verified --key "$PID" \
  --method "reproducible-rebuild:program-id" \
  --evidence-cid "$EVIDENCE_CID" --verifier "$VERIFIER"
```

The guest rejects self-verification: `verifier` must differ from the entry's
`author`. On success the entry carries `source_verified: true`, the verifier
account, the method string, and the evidence CID; readers can then filter with
`verified=true` (or `client.search({ verified: true })`). Internally,
`is_independently_verified()` requires **both** `source_verified` and a
non-empty `verifier`.

**Important:** changing the source via `update` — `code_cid`, `commit`,
`idl_cid`, or `build_config` — clears any prior attestation. Re-run the clean
rebuild and `mark-verified` after every source-changing update. Metadata-only
updates keep the attestation.

## Quick checklist

- [ ] Program deployed; 32-byte program ID recorded in hex and base58.
- [ ] Entry PDA derived with `--key <program-id>`; index PDA is `registry_index`.
- [ ] IDL + source uploaded via `tools/qtro_storage_upload.py`; CIDs in hand.
- [ ] `register` includes commit and a reproducible `build_config`; tags set.
- [ ] `/entry/<base58>`, `/search?tag=...` and `/search?name=...` return the expected record.
- [ ] A different account rebuilt from source and called `mark-verified`.
- [ ] Any source-changing update was followed by a fresh attestation.

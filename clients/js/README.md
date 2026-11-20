<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->

# @lp0023/registry-client

Small, dependency-free ESM client for the read-only [LP-0023](../../LP0023.md)
LEZ Program Registry JSON API served by `registry-api` on
`http://127.0.0.1:3030`. Requires Node.js 18+ (global `fetch`).

## Install

From a checkout of this repository:

```bash
npm install ./clients/js
```

Or add the local package to your project's `package.json`:

```json
{ "dependencies": { "@lp0023/registry-client": "file:../registry/clients/js" } }
```

No runtime dependencies are installed.

## Usage

```js
import { RegistryClient, RegistryApiError } from '@lp0023/registry-client';

const client = new RegistryClient('http://127.0.0.1:3030');

const health = await client.health();            // { status: 'ok' }
const ids = await client.index();                // base58 program IDs
const all = await client.entries();              // RegistryEntry[]

// Repeatable tags are OR-matched (case-insensitive substring).
const entries = await client.search({
  tags: ['registry', 'spel'],
  verified: true,
  limit: 10,
});

const one = await client.entry('6FMaVQ6CmbAVNHgwsmmeBe5kiiKkQRFtqu4GUQ2vAf2T');
console.log(one.name, one.version, one.idl_cid, one.source_verified);

try {
  await client.entry('not-registered');
} catch (err) {
  if (err instanceof RegistryApiError) {
    console.error(err.status, err.body); // 404, { error: '...' }
  } else {
    throw err;
  }
}
```

`RegistryApiError` carries `status` (0 for a failed `fetch`) and the parsed
response `body` (JSON when possible, otherwise the raw text).

Keys (`program_id`, `author`, `verifier`) are exposed as base58 strings. The
client accepts base58, `Public/`-prefixed base58, 64-char hex, or JSON byte
arrays from the API and normalises them. Query strings are built with
`URLSearchParams`, and path segments with `encodeURIComponent`, so CIDs and
other values containing `*` or other reserved characters are handled safely.

TypeScript users get `RegistryEntry`, `RegistrySearchOptions` (alias
`SearchOptions`), `HealthResponse`, `RegistryClient`, and `RegistryApiError`
from the package root.

## API surface

| Method | Endpoint |
| --- | --- |
| `health()` | `GET /health` |
| `index()` | `GET /index` |
| `entries()` | `GET /entries` |
| `search({ tags?, author?, verified?, limit? })` | `GET /search` |
| `entry(programId)` | `GET /entry/:program_id` |

## Development

```bash
npm test                 # node --test, no dependencies
npm run build            # tsc -p tsconfig.json (needs TypeScript installed)
```

`dist/index.js` and `dist/index.d.ts` are committed so the package works and
its tests run without installing TypeScript. If `tsc` is available, `npm run
build` regenerates them from `src/index.ts`; keep them in sync with the source.

Dual-licensed under MIT OR Apache-2.0.

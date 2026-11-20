// SPDX-License-Identifier: MIT OR Apache-2.0

import test, { afterEach } from 'node:test';
import assert from 'node:assert/strict';
import { RegistryApiError, RegistryClient } from '../dist/index.js';

const originalFetch = globalThis.fetch;
afterEach(() => {
  globalThis.fetch = originalFetch;
});

function mockJson(body, status = 200) {
  const calls = [];
  globalThis.fetch = async (url) => {
    calls.push(url);
    return {
      ok: status >= 200 && status < 300,
      status,
      text: async () => (typeof body === 'string' ? body : JSON.stringify(body)),
    };
  };
  return calls;
}

const PROGRAM_ID_BYTES = [
  77, 247, 213, 57, 107, 104, 161, 106, 174, 109, 15, 118, 142, 244, 71, 160,
  5, 94, 180, 56, 102, 130, 113, 84, 247, 17, 57, 228, 134, 38, 81, 68,
];
const PROGRAM_ID_BASE58 = '6FMaVQ6CmbAVNHgwsmmeBe5kiiKkQRFtqu4GUQ2vAf2T';

function sampleEntry() {
  return {
    program_id: PROGRAM_ID_BYTES,
    name: 'demo',
    version: '0.1.0',
    author: PROGRAM_ID_BYTES,
    description: 'integration test entry',
    tags: 'registry,spel',
    idl_cid: 'zDv*8iXZ6U',
    code_cid: 'zDv*yXuozL',
    commit: 'abc1234',
    build_config: 'guest=local,risc0=3.0.5',
    timestamp: 1_700_000_000,
    deployer_claimed: true,
    source_verified: true,
    verifier: PROGRAM_ID_BYTES,
    verification_method: 'reproducible-rebuild:program-id',
    evidence_cid: 'zDv*8iXZ6U',
  };
}

test('health() requests /health on the default base URL', async () => {
  const calls = mockJson({ status: 'ok' });
  assert.deepEqual(await new RegistryClient().health(), { status: 'ok' });
  assert.equal(calls[0].toString(), 'http://127.0.0.1:3030/health');
});

test('a base URL with a path prefix is preserved', async () => {
  const calls = mockJson([]);
  await new RegistryClient('http://example.test/api').index();
  assert.equal(calls[0].toString(), 'http://example.test/api/index');
});

test('search() repeats tag parameters and escapes the rest', async () => {
  const calls = mockJson([]);
  await new RegistryClient().search({
    tags: ['registry', 'wallet & defi'],
    author: PROGRAM_ID_BASE58,
    verified: true,
    limit: 5,
  });

  const url = calls[0];
  assert.equal(url.pathname, '/search');
  assert.deepEqual(url.searchParams.getAll('tag'), ['registry', 'wallet & defi']);
  assert.equal(url.searchParams.get('author'), PROGRAM_ID_BASE58);
  assert.equal(url.searchParams.get('verified'), 'true');
  assert.equal(url.searchParams.get('limit'), '5');
  assert.match(url.search, /%26/); // URLSearchParams escaped the '&'
});

test('non-2xx responses throw RegistryApiError with status and parsed body', async () => {
  mockJson({ error: 'program id is not in the registry index' }, 404);
  await assert.rejects(() => new RegistryClient().entry('missing'), (err) => {
    assert.ok(err instanceof RegistryApiError);
    assert.equal(err.status, 404);
    assert.deepEqual(err.body, { error: 'program id is not in the registry index' });
    assert.match(err.message, /404/);
    return true;
  });
});

test('non-JSON error bodies are kept as text', async () => {
  mockJson('upstream unavailable', 502);
  await assert.rejects(() => new RegistryClient().entries(), (err) => {
    assert.ok(err instanceof RegistryApiError);
    assert.equal(err.status, 502);
    assert.equal(err.body, 'upstream unavailable');
    return true;
  });
});

test('byte-array keys are normalised to base58', async () => {
  mockJson([sampleEntry()]);
  const [entry] = await new RegistryClient().entries();
  assert.equal(entry.program_id, PROGRAM_ID_BASE58);
  assert.equal(entry.author, PROGRAM_ID_BASE58);
  assert.equal(entry.verifier, PROGRAM_ID_BASE58);
  assert.equal(entry.evidence_cid, 'zDv*8iXZ6U');
});

test('entry() URL-encodes the path segment', async () => {
  const calls = mockJson(sampleEntry());
  await new RegistryClient().entry('a/b');
  assert.equal(calls[0].pathname, '/entry/a%2Fb');
});

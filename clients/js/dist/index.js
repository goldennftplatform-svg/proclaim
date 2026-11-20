// SPDX-License-Identifier: MIT OR Apache-2.0
// Generated from src/index.ts (kept in sync by hand; see README.md).

const BASE58_ALPHABET = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';

function encodeBase58(bytes) {
  let leadingZeros = 0;
  while (leadingZeros < bytes.length && bytes[leadingZeros] === 0) leadingZeros++;
  if (leadingZeros === bytes.length) return '1'.repeat(bytes.length);

  const digits = [0];
  for (let i = leadingZeros; i < bytes.length; i++) {
    let carry = bytes[i];
    for (let j = 0; j < digits.length; j++) {
      carry += digits[j] << 8;
      digits[j] = carry % 58;
      carry = Math.floor(carry / 58);
    }
    while (carry > 0) {
      digits.push(carry % 58);
      carry = Math.floor(carry / 58);
    }
  }

  let out = '1'.repeat(leadingZeros);
  for (let i = digits.length - 1; i >= 0; i--) out += BASE58_ALPHABET[digits[i]];
  return out;
}

function hexToBytes(hex) {
  const out = [];
  for (let i = 0; i < hex.length; i += 2) {
    out.push(Number.parseInt(hex.slice(i, i + 2), 16));
  }
  return out;
}

function normalizeKey(value) {
  if (Array.isArray(value)) return encodeBase58(value);
  const key = value.startsWith('Public/') ? value.slice('Public/'.length) : value;
  return /^[0-9a-fA-F]{64}$/.test(key) ? encodeBase58(hexToBytes(key)) : key;
}

function normalizeEntry(raw) {
  return {
    ...raw,
    program_id: normalizeKey(raw.program_id),
    author: normalizeKey(raw.author),
    verifier: raw.verifier === null || raw.verifier === undefined ? null : normalizeKey(raw.verifier),
  };
}

export class RegistryApiError extends Error {
  constructor(status, body, message) {
    super(message);
    this.name = 'RegistryApiError';
    this.status = status;
    this.body = body;
  }
}

export class RegistryClient {
  constructor(baseUrl = 'http://127.0.0.1:3030') {
    const url = new URL(baseUrl);
    if (!url.pathname.endsWith('/')) url.pathname += '/';
    this.baseUrl = url;
  }

  async get(path, params) {
    const url = new URL(path, this.baseUrl);
    if (params) url.search = params.toString();

    let response;
    try {
      response = await fetch(url);
    } catch (cause) {
      const message = cause instanceof Error ? cause.message : String(cause);
      throw new RegistryApiError(0, { error: message }, `Registry API request failed: ${url} (${message})`);
    }

    const text = await response.text();
    let body = text;
    if (text.length > 0) {
      try {
        body = JSON.parse(text);
      } catch {
        // Keep the raw text for non-JSON responses.
      }
    }

    if (!response.ok) {
      const detail =
        typeof body === 'object' && body !== null && 'error' in body
          ? String(body.error)
          : text;
      const suffix = detail ? `: ${detail}` : '';
      throw new RegistryApiError(response.status, body, `Registry API ${response.status} at ${url}${suffix}`);
    }

    return body;
  }

  async health() {
    return this.get('health');
  }

  async index() {
    const ids = await this.get('index');
    return ids.map(normalizeKey);
  }

  async entries() {
    const entries = await this.get('entries');
    return entries.map(normalizeEntry);
  }

  async search(options = {}) {
    const params = new URLSearchParams();
    for (const tag of options.tags ?? []) params.append('tag', tag);
    if (options.author !== undefined) params.set('author', normalizeKey(options.author));
    if (options.verified !== undefined) params.set('verified', String(options.verified));
    if (options.limit !== undefined) params.set('limit', String(options.limit));

    const entries = await this.get('search', params);
    return entries.map(normalizeEntry);
  }

  async entry(programId) {
    const raw = await this.get(`entry/${encodeURIComponent(normalizeKey(programId))}`);
    return normalizeEntry(raw);
  }
}

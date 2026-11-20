// SPDX-License-Identifier: MIT AND Apache-2.0

/**
 * Dependency-free ESM client for the LP-0023 LEZ Program Registry read-only
 * JSON API (`registry-api`, default http://127.0.0.1:3030).
 *
 * Byte-array keys are normalised to base58 so callers see one shape whether
 * the server serialises `[u8; 32]` as a JSON array of bytes or as base58.
 */

/** A registry entry as returned by this client (keys are base58 strings). */
export interface RegistryEntry {
  /** Base58-encoded 32-byte program ID (also the entry PDA seed). */
  program_id: string;
  name: string;
  version: string;
  /** Base58-encoded 32-byte account ID of the original author. */
  author: string;
  description: string;
  /** Comma-separated tags; the API matches them with OR semantics. */
  tags: string;
  /** Logos Storage CID of the IDL document. */
  idl_cid: string;
  /** Logos Storage CID of the source archive. */
  code_cid: string;
  /** Git commit that produced the deployed ELF. */
  commit: string;
  /** Build settings needed to reproduce the ELF. */
  build_config: string;
  /** Registration time, Unix seconds. */
  timestamp: number;
  /** Signer claimed to be the deployer (not an independent attestation). */
  deployer_claimed: boolean;
  /** An independent rebuild matched the deployed bytecode. */
  source_verified: boolean;
  /** Base58 verifier account, or null while unverified. */
  verifier: string | null;
  verification_method: string;
  evidence_cid: string;
}

/** Filters accepted by `RegistryClient.search`. */
export interface RegistrySearchOptions {
  /** Case-insensitive substring match against the entry name. */
  name?: string;
  /** Repeatable tags; OR-matched, case-insensitive substring match. */
  tags?: string[];
  /** Author account as base58, hex, or 32 bytes. */
  author?: string | number[];
  /** When true, return only independently verified entries. */
  verified?: boolean;
  /** Maximum number of entries to return. */
  limit?: number;
}

/** Short alias for {@link RegistrySearchOptions}. */
export type SearchOptions = RegistrySearchOptions;

/** Raw key form accepted on the wire: base58/hex string or JSON byte array. */
type WireKey = string | number[];

interface WireRegistryEntry {
  program_id: WireKey;
  name: string;
  version: string;
  author: WireKey;
  description: string;
  tags: string;
  idl_cid: string;
  code_cid: string;
  commit: string;
  build_config: string;
  timestamp: number;
  deployer_claimed: boolean;
  source_verified: boolean;
  verifier: WireKey | null;
  verification_method: string;
  evidence_cid: string;
}

interface HealthResponse {
  status: string;
}

const BASE58_ALPHABET = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';

function encodeBase58(bytes: readonly number[]): string {
  let leadingZeros = 0;
  while (leadingZeros < bytes.length && bytes[leadingZeros] === 0) leadingZeros++;
  if (leadingZeros === bytes.length) return '1'.repeat(bytes.length);

  const digits: number[] = [0];
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

function hexToBytes(hex: string): number[] {
  const out: number[] = [];
  for (let i = 0; i < hex.length; i += 2) {
    out.push(Number.parseInt(hex.slice(i, i + 2), 16));
  }
  return out;
}

function normalizeKey(value: WireKey): string {
  if (Array.isArray(value)) return encodeBase58(value);
  const key = value.startsWith('Public/') ? value.slice('Public/'.length) : value;
  return /^[0-9a-fA-F]{64}$/.test(key) ? encodeBase58(hexToBytes(key)) : key;
}

function normalizeEntry(raw: WireRegistryEntry): RegistryEntry {
  return {
    ...raw,
    program_id: normalizeKey(raw.program_id),
    author: normalizeKey(raw.author),
    verifier: raw.verifier === null || raw.verifier === undefined ? null : normalizeKey(raw.verifier),
  };
}

/**
 * Resolution state of an entry's IDL pointer. Rendering entry metadata never
 * depends on the IDL being fetchable: a missing or placeholder CID is reported
 * as `'unavailable'` while the on-chain record stays fully usable.
 */
export type IdlAvailability = 'available' | 'unavailable';

export function idlAvailability(entry: RegistryEntry): IdlAvailability {
  const cid = (entry.idl_cid ?? '').trim().toLowerCase();
  return cid === '' || cid === 'none' || cid === 'null' || cid === 'placeholder'
    ? 'unavailable'
    : 'available';
}

/** Error thrown for any non-2xx response (or a failed fetch). */
export class RegistryApiError extends Error {
  readonly status: number;
  readonly body: unknown;

  constructor(status: number, body: unknown, message: string) {
    super(message);
    this.name = 'RegistryApiError';
    this.status = status;
    this.body = body;
  }
}

/** Minimal client for the read-only registry endpoints. */
export class RegistryClient {
  private readonly baseUrl: URL;

  constructor(baseUrl: string | URL = 'http://127.0.0.1:3030') {
    const url = new URL(baseUrl);
    if (!url.pathname.endsWith('/')) url.pathname += '/';
    this.baseUrl = url;
  }

  private async get<T>(path: string, params?: URLSearchParams): Promise<T> {
    const url = new URL(path, this.baseUrl);
    if (params) url.search = params.toString();

    let response: Response;
    try {
      response = await fetch(url);
    } catch (cause) {
      const message = cause instanceof Error ? cause.message : String(cause);
      throw new RegistryApiError(0, { error: message }, `Registry API request failed: ${url} (${message})`);
    }

    const text = await response.text();
    let body: unknown = text;
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
          ? String((body as { error: unknown }).error)
          : text;
      const suffix = detail ? `: ${detail}` : '';
      throw new RegistryApiError(response.status, body, `Registry API ${response.status} at ${url}${suffix}`);
    }

    return body as T;
  }

  /** GET /health — liveness probe. */
  async health(): Promise<HealthResponse> {
    return this.get<HealthResponse>('health');
  }

  /** GET /index — registered program IDs, base58-encoded. */
  async index(): Promise<string[]> {
    const ids = await this.get<WireKey[]>('index');
    return ids.map(normalizeKey);
  }

  /** GET /entries — every decoded entry. */
  async entries(): Promise<RegistryEntry[]> {
    const entries = await this.get<WireRegistryEntry[]>('entries');
    return entries.map(normalizeEntry);
  }

  /** GET /search — repeat `tag` for OR filtering; URLSearchParams does the escaping. */
  async search(options: RegistrySearchOptions = {}): Promise<RegistryEntry[]> {
    const params = new URLSearchParams();
    if (options.name !== undefined) params.set('name', options.name);
    for (const tag of options.tags ?? []) params.append('tag', tag);
    if (options.author !== undefined) params.set('author', normalizeKey(options.author));
    if (options.verified !== undefined) params.set('verified', String(options.verified));
    if (options.limit !== undefined) params.set('limit', String(options.limit));

    const entries = await this.get<WireRegistryEntry[]>('search', params);
    return entries.map(normalizeEntry);
  }

  /** GET /entry/:program_id — 404 is surfaced as a `RegistryApiError`. */
  async entry(programId: string | number[]): Promise<RegistryEntry> {
    const raw = await this.get<WireRegistryEntry>(`entry/${encodeURIComponent(normalizeKey(programId))}`);
    return normalizeEntry(raw);
  }
}

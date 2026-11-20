// SPDX-License-Identifier: MIT OR Apache-2.0
// Generated from src/index.ts (kept in sync by hand; see README.md).

/** A registry entry as returned by this client (keys are base58 strings). */
export interface RegistryEntry {
  program_id: string;
  name: string;
  version: string;
  author: string;
  description: string;
  tags: string;
  idl_cid: string;
  code_cid: string;
  commit: string;
  build_config: string;
  timestamp: number;
  deployer_claimed: boolean;
  source_verified: boolean;
  verifier: string | null;
  verification_method: string;
  evidence_cid: string;
}

/** Filters accepted by `RegistryClient.search`. */
export interface RegistrySearchOptions {
  tags?: string[];
  author?: string | number[];
  verified?: boolean;
  limit?: number;
}

export type SearchOptions = RegistrySearchOptions;

export interface HealthResponse {
  status: string;
}

/** Error thrown for any non-2xx response (or a failed fetch). */
export declare class RegistryApiError extends Error {
  readonly status: number;
  readonly body: unknown;
  constructor(status: number, body: unknown, message: string);
}

/** Minimal client for the read-only registry endpoints. */
export declare class RegistryClient {
  private readonly baseUrl;
  constructor(baseUrl?: string | URL);
  health(): Promise<HealthResponse>;
  index(): Promise<string[]>;
  entries(): Promise<RegistryEntry[]>;
  search(options?: RegistrySearchOptions): Promise<RegistryEntry[]>;
  entry(programId: string | number[]): Promise<RegistryEntry>;
}

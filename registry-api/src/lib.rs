//! Shared host-side client helpers for the LP-0023 LEZ Program Registry.
//!
//! Keeps chain access, PDA derivation, decoding and filtering in one place so
//! `registry-api` and `registry_browser` cannot drift apart.
//! Dual-licensed: MIT AND Apache-2.0.

use std::fmt;

use base58::{FromBase58, ToBase58};
use nssa::AccountId;
use nssa_core::program::{PdaSeed, ProgramId};
use registry_core::{search_entries_with_name, RegistryEntry, RegistryIndex};
use tokio::sync::{mpsc, oneshot};
use wallet::WalletCore;

/// Final v5 deployed registry program ID (big-endian hex).
pub const DEFAULT_PROGRAM_ID_HEX: &str =
    "4df7d5396b68a16aae6d0f768ef447a0055eb43866827154f71139e486265144";
/// Const PDA seed used by `init_index` / `search` (`registry-idl.json`).
pub const INDEX_SEED: &[u8] = b"registry_index";
/// Wallet home relative to the current directory when the env var is unset.
pub const DEFAULT_WALLET_HOME: &str = ".scaffold/wallet";

/// Error returned by the shared registry client helpers.
#[derive(Debug, Clone)]
pub struct RegistryClientError {
    message: String,
}

impl RegistryClientError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for RegistryClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for RegistryClientError {}

pub type RegistryClientResult<T> = Result<T, RegistryClientError>;

fn err(message: impl Into<String>) -> RegistryClientError {
    RegistryClientError::new(message)
}

/// Parse a 64-char hex program ID into the LE `u32` words used by LEZ.
pub fn program_id_from_hex(s: &str) -> RegistryClientResult<ProgramId> {
    let s = s.trim_start_matches("0x");
    if s.len() != 64 {
        return Err(err(format!(
            "program id hex must be 64 chars, got {}",
            s.len()
        )));
    }
    let mut pid = [0u32; 8];
    for (i, chunk) in s.as_bytes().chunks(8).enumerate() {
        let word = std::str::from_utf8(chunk)
            .ok()
            .and_then(|x| u32::from_str_radix(x, 16).ok())
            .ok_or_else(|| err("invalid program id hex".to_string()))?;
        pid[i] = word.swap_bytes(); // hex string is big-endian; ProgramId is LE u32 words
    }
    Ok(pid)
}

/// Decode a 32-byte key from base58 (optionally `Public/`-prefixed) or 64-char hex.
pub fn parse_key32(s: &str) -> RegistryClientResult<[u8; 32]> {
    let s = s.trim_start_matches("Public/").trim_start_matches("0x");
    if s.len() == 64 {
        let mut out = [0u8; 32];
        for (i, chunk) in s.as_bytes().chunks(2).enumerate() {
            let h = std::str::from_utf8(chunk)
                .ok()
                .and_then(|x| u8::from_str_radix(x, 16).ok())
                .ok_or_else(|| err(format!("invalid hex key: {}", s)))?;
            out[i] = h;
        }
        return Ok(out);
    }
    let bytes = s
        .from_base58()
        .map_err(|_| err(format!("invalid base58 key: {}", s)))?;
    let len = bytes.len();
    bytes
        .try_into()
        .map_err(|_| err(format!("key must be 32 bytes, got {}", len)))
}

/// Base58 encoding used for all public program/author IDs.
pub fn to_base58(key: &[u8; 32]) -> String {
    key.to_base58()
}

fn const_seed(s: &[u8]) -> RegistryClientResult<[u8; 32]> {
    if s.len() > 32 {
        return Err(err("const seed exceeds 32 bytes"));
    }
    let mut out = [0u8; 32];
    out[..s.len()].copy_from_slice(s);
    Ok(out)
}

/// PDA of the global registry index for a program ID.
pub fn index_pda(program_id: &ProgramId) -> RegistryClientResult<AccountId> {
    let seed = const_seed(INDEX_SEED)?;
    Ok(AccountId::for_public_pda(program_id, &PdaSeed::new(seed)))
}

/// PDA of a single registry entry, seeded by its 32-byte program ID.
pub fn entry_pda(program_id: &ProgramId, key: &[u8; 32]) -> AccountId {
    AccountId::for_public_pda(program_id, &PdaSeed::new(*key))
}

/// Fetch raw account data (via the configured sequencer) for an account ID.
pub async fn fetch_account_data(wc: &WalletCore, id: AccountId) -> RegistryClientResult<Vec<u8>> {
    let acc = wc
        .get_account_public(id)
        .await
        .map_err(|e| err(format!("failed to fetch {}: {}", id, e)))?;
    if acc.data.is_empty() {
        return Err(err(format!("{} has no data", id)));
    }
    Ok(acc.data.to_vec())
}

/// Fetch and Borsh-decode the global registry index.
pub async fn fetch_index(
    wc: &WalletCore,
    program_id: &ProgramId,
) -> RegistryClientResult<RegistryIndex> {
    let id = index_pda(program_id)?;
    let bytes = fetch_account_data(wc, id).await?;
    RegistryIndex::from_account_data(&bytes)
        .map_err(|e| err(format!("failed to decode RegistryIndex: {}", e)))
}

/// Fetch and Borsh-decode a single registry entry by its 32-byte program ID.
pub async fn fetch_entry(
    wc: &WalletCore,
    program_id: &ProgramId,
    key: &[u8; 32],
) -> RegistryClientResult<RegistryEntry> {
    let id = entry_pda(program_id, key);
    let bytes = fetch_account_data(wc, id).await?;
    RegistryEntry::from_account_data(&bytes)
        .map_err(|e| err(format!("failed to decode RegistryEntry: {}", e)))
}

/// Fetch every entry referenced by the index. Unreadable entries are skipped
/// (with a warning on stderr) so one bad account cannot break enumeration.
pub async fn fetch_entries(
    wc: &WalletCore,
    program_id: &ProgramId,
    index: &RegistryIndex,
) -> RegistryClientResult<Vec<RegistryEntry>> {
    let mut entries = Vec::with_capacity(index.program_ids.len());
    for key in &index.program_ids {
        let id = entry_pda(program_id, key);
        match fetch_account_data(wc, id).await {
            Ok(data) => match RegistryEntry::from_account_data(&data) {
                Ok(entry) => entries.push(entry),
                Err(e) => eprintln!("warn: failed to decode entry {}: {}", id, e),
            },
            Err(e) => eprintln!("warn: {}", e),
        }
    }
    Ok(entries)
}

/// Filter entries by name (case-insensitive substring), tag (OR semantics),
/// author and independent verification.
pub fn filter_entries<'a>(
    entries: &'a [RegistryEntry],
    tags: &[String],
    author: Option<&[u8; 32]>,
    verified_only: bool,
    name: Option<&str>,
) -> Vec<&'a RegistryEntry> {
    let tag_refs: Vec<&str> = tags.iter().map(String::as_str).collect();
    search_entries_with_name(entries, &tag_refs, author, verified_only, name)
}

/// A request to the wallet-owning worker thread.
#[derive(Debug)]
enum WorkerRequest {
    Index {
        program_id: ProgramId,
        reply: oneshot::Sender<RegistryClientResult<RegistryIndex>>,
    },
    Entries {
        program_id: ProgramId,
        index: RegistryIndex,
        reply: oneshot::Sender<RegistryClientResult<Vec<RegistryEntry>>>,
    },
    Entry {
        program_id: ProgramId,
        key: [u8; 32],
        reply: oneshot::Sender<RegistryClientResult<RegistryEntry>>,
    },
}

/// Cheap, cloneable handle to a single `WalletCore` owned by a dedicated
/// current-thread runtime.
///
/// `WalletCore::get_account_public` builds a future that is not `Send` for all
/// lifetimes, so it cannot be awaited directly inside an axum handler. Queries
/// are sent over a channel to the worker thread, which runs the future on a
/// local runtime and returns a plain `Send` result.
#[derive(Clone)]
pub struct RegistryService {
    tx: mpsc::Sender<WorkerRequest>,
}

impl RegistryService {
    /// Initialize the wallet and start its worker thread.
    pub async fn start() -> RegistryClientResult<Self> {
        let (tx, mut rx) = mpsc::channel(64);
        let (ready_tx, ready_rx) = oneshot::channel::<RegistryClientResult<()>>();

        std::thread::Builder::new()
            .name("registry-wallet".to_string())
            .spawn(move || {
                let runtime = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(rt) => rt,
                    Err(e) => {
                        let _ = ready_tx
                            .send(Err(err(format!("failed to build wallet runtime: {}", e))));
                        return;
                    }
                };
                runtime.block_on(async move {
                    let wc = match WalletCore::from_env().await {
                        Ok(wc) => wc,
                        Err(e) => {
                            let _ =
                                ready_tx.send(Err(err(format!("failed to init wallet: {}", e))));
                            return;
                        }
                    };
                    let _ = ready_tx.send(Ok(()));
                    while let Some(request) = rx.recv().await {
                        match request {
                            WorkerRequest::Index { program_id, reply } => {
                                let _ = reply.send(fetch_index(&wc, &program_id).await);
                            }
                            WorkerRequest::Entries {
                                program_id,
                                index,
                                reply,
                            } => {
                                let _ = reply.send(fetch_entries(&wc, &program_id, &index).await);
                            }
                            WorkerRequest::Entry {
                                program_id,
                                key,
                                reply,
                            } => {
                                let _ = reply.send(fetch_entry(&wc, &program_id, &key).await);
                            }
                        }
                    }
                });
            })
            .map_err(|e| err(format!("failed to spawn wallet worker: {}", e)))?;

        ready_rx
            .await
            .map_err(|e| err(format!("wallet worker initialization failed: {}", e)))??;
        Ok(Self { tx })
    }

    pub async fn index(&self, program_id: ProgramId) -> RegistryClientResult<RegistryIndex> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .send(WorkerRequest::Index { program_id, reply })
            .await
            .map_err(|_| err("wallet worker is unavailable"))?;
        rx.await
            .map_err(|e| err(format!("wallet worker dropped the request: {}", e)))?
    }

    pub async fn entries(
        &self,
        program_id: ProgramId,
        index: RegistryIndex,
    ) -> RegistryClientResult<Vec<RegistryEntry>> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .send(WorkerRequest::Entries {
                program_id,
                index,
                reply,
            })
            .await
            .map_err(|_| err("wallet worker is unavailable"))?;
        rx.await
            .map_err(|e| err(format!("wallet worker dropped the request: {}", e)))?
    }

    pub async fn entry(
        &self,
        program_id: ProgramId,
        key: [u8; 32],
    ) -> RegistryClientResult<RegistryEntry> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .send(WorkerRequest::Entry {
                program_id,
                key,
                reply,
            })
            .await
            .map_err(|_| err("wallet worker is unavailable"))?;
        rx.await
            .map_err(|e| err(format!("wallet worker dropped the request: {}", e)))?
    }
}

#![no_main]

//! LP-0023 — LEZ Program Registry.
//! One PDA per program ID. Metadata + Logos Storage pointers live on-chain.
//! Dual-licensed: MIT AND Apache-2.0.

use spel_framework::prelude::*;
use nssa_core::account::Data;

risc0_zkvm::guest::entry!(main);

const ERR_SERIALIZE: u32 = 9001;
const ERR_DESERIALIZE: u32 = 9002;
const ERR_UNAUTHORIZED: u32 = 9003;

#[lez_program]
mod registry {
    #[allow(unused_imports)]
    use super::*;

    /// On-chain registry record. `#[account_type]` makes it decodable by
    /// `spel inspect <PDA> --type RegistryEntry` and by generated clients.
    #[derive(BorshSerialize, BorshDeserialize)]
    #[account_type]
    pub struct RegistryEntry {
        /// The 32-byte program ID this entry describes (also the PDA seed).
        pub program_id: [u8; 32],
        /// Human-readable program name.
        pub name: String,
        /// Semantic version of the deployed program.
        pub version: String,
        /// Account ID of the registering author.
        pub author: [u8; 32],
        /// Free-text description.
        pub description: String,
        /// Comma-separated search tags.
        pub tags: String,
        /// Logos Storage CID of the generated IDL document.
        pub idl_cid: String,
        /// Logos Storage CID of the source archive.
        pub code_cid: String,
        /// Source commit that produced the deployed ELF.
        pub commit: String,
        /// Build configuration needed to reproduce the ELF.
        pub build_config: String,
        /// Registration time (caller-supplied Unix seconds; guest has no clock).
        pub timestamp: u64,
        /// True when the signer claims to be the program's deployer.
        pub deployer_claimed: bool,
        /// True after an independent rebuild matched the deployed bytecode.
        pub source_verified: bool,
        /// Account that performed the independent rebuild (None until verified).
        pub verifier: Option<[u8; 32]>,
        /// Verification method, e.g. "reproducible-rebuild:program-id".
        pub verification_method: String,
        /// Logos Storage CID of the rebuild log / attestation.
        pub evidence_cid: String,
    }

    fn encode(entry: &RegistryEntry) -> Result<Data, SpelError> {
        let bytes = borsh::to_vec(entry)
            .map_err(|e| SpelError::custom(ERR_SERIALIZE, format!("borsh error: {e}")))?;
        Data::try_from(bytes)
            .map_err(|_| SpelError::custom(ERR_SERIALIZE, "registry entry exceeds account data limit"))
    }

    fn decode(account: &AccountWithMetadata) -> Result<RegistryEntry, SpelError> {
        let bytes: Vec<u8> = account.account.data.clone().into();
        borsh::from_slice(&bytes)
            .map_err(|e| SpelError::custom(ERR_DESERIALIZE, format!("borsh error: {e}")))
    }

    /// Global index of registered program IDs, stored at a fixed PDA.
    /// Enables `search` without an external indexer.
    #[derive(BorshSerialize, BorshDeserialize)]
    #[account_type]
    pub struct RegistryIndex {
        pub program_ids: Vec<[u8; 32]>,
    }

    fn decode_index(account: &AccountWithMetadata) -> Result<RegistryIndex, SpelError> {
        let bytes: Vec<u8> = account.account.data.clone().into();
        borsh::from_slice(&bytes)
            .map_err(|e| SpelError::custom(ERR_DESERIALIZE, format!("borsh error: {e}")))
    }

    fn encode_index(index: &RegistryIndex) -> Result<Data, SpelError> {
        let bytes = borsh::to_vec(index)
            .map_err(|e| SpelError::custom(ERR_SERIALIZE, format!("borsh error: {e}")))?;
        Data::try_from(bytes)
            .map_err(|_| SpelError::custom(ERR_SERIALIZE, "registry index exceeds account data limit"))
    }

    /// Create the global index PDA. Call once before the first `register`.
    #[instruction]
    pub fn init_index(
        #[account(init, pda = literal("registry_index"))] mut index: AccountWithMetadata,
        #[account(signer)] authority: AccountWithMetadata,
    ) -> SpelResult {
        index.account.data = encode_index(&RegistryIndex { program_ids: vec![] })?;
        Ok(SpelOutput::execute(vec![index, authority], vec![]))
    }

    /// Register a program. Entry PDA is derived from its program ID; the
    /// program ID is appended to the global index for `search`.
    #[instruction]
    pub fn register(
        #[account(init, pda = arg("key"))] mut entry: AccountWithMetadata,
        #[account(mut, pda = literal("registry_index"))] mut index: AccountWithMetadata,
        #[account(signer)] author: AccountWithMetadata,
        key: [u8; 32],
        name: String,
        version: String,
        description: String,
        tags: String,
        idl_cid: String,
        code_cid: String,
        commit: String,
        build_config: String,
        timestamp: u64,
        deployer_claimed: bool,
    ) -> SpelResult {
        let record = RegistryEntry {
            program_id: key,
            name,
            version,
            author: *author.account_id.value(),
            description,
            tags,
            idl_cid,
            code_cid,
            commit,
            build_config,
            timestamp,
            deployer_claimed,
            source_verified: false,
            verifier: None,
            verification_method: String::new(),
            evidence_cid: String::new(),
        };
        entry.account.data = encode(&record)?;

        let mut registry_index = decode_index(&index)?;
        if !registry_index.program_ids.contains(&key) {
            registry_index.program_ids.push(key);
            index.account.data = encode_index(&registry_index)?;
        }
        Ok(SpelOutput::execute(vec![entry, index, author], vec![]))
    }

    /// Update an existing entry. Only the original registering author may do so.
    #[instruction]
    pub fn update(
        #[account(mut, pda = arg("key"))] mut entry: AccountWithMetadata,
        #[account(signer)] author: AccountWithMetadata,
        key: [u8; 32],
        version: String,
        description: String,
        tags: String,
        idl_cid: String,
        code_cid: String,
        commit: String,
        build_config: String,
    ) -> SpelResult {
        let mut record = decode(&entry)?;
        if *author.account_id.value() != record.author {
            return Err(SpelError::custom(
                ERR_UNAUTHORIZED,
                "only the original registering author may update",
            ));
        }
        let _ = key; // PDA seed already validated by the macro

        // A verification attests to a specific source snapshot. Changing any
        // source-identifying field invalidates it; metadata-only edits do not.
        let source_changed = idl_cid != record.idl_cid
            || code_cid != record.code_cid
            || commit != record.commit
            || build_config != record.build_config;
        if source_changed {
            record.source_verified = false;
            record.verifier = None;
            record.verification_method.clear();
            record.evidence_cid.clear();
        }

        record.version = version;
        record.description = description;
        record.tags = tags;
        record.idl_cid = idl_cid;
        record.code_cid = code_cid;
        record.commit = commit;
        record.build_config = build_config;
        entry.account.data = encode(&record)?;
        Ok(SpelOutput::execute(vec![entry, author], vec![]))
    }

    /// Record an independent source-verification attestation. The rebuild is
    /// performed off-chain and must reproduce the on-chain program ID; the
    /// verifier's identity and evidence CID are stored on-chain. The registering
    /// author may not verify their own entry.
    #[instruction]
    pub fn mark_verified(
        #[account(mut, pda = arg("key"))] mut entry: AccountWithMetadata,
        #[account(signer)] verifier: AccountWithMetadata,
        key: [u8; 32],
        method: String,
        evidence_cid: String,
    ) -> SpelResult {
        let mut record = decode(&entry)?;
        let _ = key; // PDA seed already validated by the macro
        if record.program_id != key {
            return Err(SpelError::custom(ERR_UNAUTHORIZED, "entry program ID mismatch"));
        }
        let verifier_id = *verifier.account_id.value();
        if verifier_id == record.author {
            return Err(SpelError::custom(
                ERR_UNAUTHORIZED,
                "the registering author cannot verify their own entry",
            ));
        }
        record.source_verified = true;
        record.verifier = Some(verifier_id);
        record.verification_method = method;
        record.evidence_cid = evidence_cid;
        entry.account.data = encode(&record)?;
        Ok(SpelOutput::execute(vec![entry, verifier], vec![]))
    }

    /// Read-only lookup by program ID. The CLI derives the same PDA from
    /// `program_id`; clients decode the account with the `RegistryEntry` IDL type.
    #[instruction]
    pub fn lookup(
        #[account(pda = arg("key"))] entry: AccountWithMetadata,
        key: [u8; 32],
    ) -> SpelResult {
        let _ = key;
        Ok(SpelOutput::execute(vec![entry], vec![]))
    }

    /// Read the global index. Clients enumerate the returned program IDs,
    /// derive each entry PDA, and filter by tags/author/verification locally.
    #[instruction]
    pub fn search(
        #[account(pda = literal("registry_index"))] index: AccountWithMetadata,
    ) -> SpelResult {
        Ok(SpelOutput::execute(vec![index], vec![]))
    }
}

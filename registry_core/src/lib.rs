//! Shared, host-side types for the LP-0023 LEZ Program Registry.
//! Layout must stay byte-identical to the guest `RegistryEntry` (Borsh).
//! Dual-licensed: MIT OR Apache-2.0.

use serde::{Deserialize, Serialize};

/// On-chain registry record, mirrored from `methods/guest/src/bin/registry.rs`.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    borsh::BorshSerialize,
    borsh::BorshDeserialize,
)]
pub struct RegistryEntry {
    /// 32-byte program ID (also the PDA seed).
    pub program_id: [u8; 32],
    /// Human-readable program name.
    pub name: String,
    /// Semantic version.
    pub version: String,
    /// 32-byte account ID of the registering author.
    pub author: [u8; 32],
    /// Free-text description.
    pub description: String,
    /// Comma-separated search tags.
    pub tags: String,
    /// Logos Storage CID of the IDL document.
    pub idl_cid: String,
    /// Logos Storage CID of the source archive.
    pub code_cid: String,
    /// Git commit that produced the deployed ELF.
    pub commit: String,
    /// Build config needed to reproduce the ELF.
    pub build_config: String,
    /// Registration time (Unix seconds, supplied by caller).
    pub timestamp: u64,
    /// Signer claims to be the program deployer.
    pub deployer_claimed: bool,
    /// Independent rebuild matched the deployed bytecode.
    pub source_verified: bool,
    /// Account that performed the independent rebuild (None until verified).
    pub verifier: Option<[u8; 32]>,
    /// Verification method, e.g. "reproducible-rebuild:program-id".
    pub verification_method: String,
    /// Logos Storage CID of the rebuild log / attestation.
    pub evidence_cid: String,
}

/// Global on-chain index of registered program IDs.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    borsh::BorshSerialize,
    borsh::BorshDeserialize,
)]
pub struct RegistryIndex {
    pub program_ids: Vec<[u8; 32]>,
}

impl RegistryIndex {
    pub fn from_account_data(data: &[u8]) -> Result<Self, borsh::io::Error> {
        borsh::from_slice(data)
    }

    pub fn to_account_data(&self) -> Result<Vec<u8>, borsh::io::Error> {
        borsh::to_vec(self)
    }
}

impl RegistryEntry {
    /// Decode a Borsh-serialized account data blob.
    pub fn from_account_data(data: &[u8]) -> Result<Self, borsh::io::Error> {
        borsh::from_slice(data)
    }

    /// Encode to Borsh for account data.
    pub fn to_account_data(&self) -> Result<Vec<u8>, borsh::io::Error> {
        borsh::to_vec(self)
    }

    /// True when `needle` matches one of the comma-separated tags
    /// (case-insensitive substring match).
    pub fn matches_tag(&self, needle: &str) -> bool {
        let n = needle.trim().to_ascii_lowercase();
        self.tags
            .split(',')
            .any(|t| t.trim().to_ascii_lowercase().contains(&n))
    }

    /// Match any of several tags (OR semantics).
    pub fn matches_any_tag<'a>(&self, needles: impl IntoIterator<Item = &'a str>) -> bool {
        needles.into_iter().any(|n| self.matches_tag(n))
    }

    /// True when `needle` is a case-insensitive substring of the entry name.
    pub fn matches_name(&self, needle: &str) -> bool {
        let n = needle.trim().to_ascii_lowercase();
        self.name.to_ascii_lowercase().contains(&n)
    }

    /// How a client should present this entry's IDL pointer.
    ///
    /// Decoding and displaying an entry never depends on the IDL being
    /// resolvable: a missing or placeholder pointer is surfaced as
    /// `Unavailable` while the on-chain metadata remains fully usable.
    pub fn idl_availability(&self) -> IdlAvailability {
        let cid = self.idl_cid.trim();
        if cid.is_empty()
            || cid.eq_ignore_ascii_case("none")
            || cid.eq_ignore_ascii_case("null")
            || cid.eq_ignore_ascii_case("placeholder")
        {
            IdlAvailability::Unavailable
        } else {
            IdlAvailability::Available(cid.to_string())
        }
    }

    /// True when a byte-for-byte independent rebuild has been attested by a
    /// verifier distinct from the registering author.
    pub fn is_independently_verified(&self) -> bool {
        self.source_verified && self.verifier.is_some()
    }
}

/// Resolution state of an entry's IDL pointer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdlAvailability {
    /// A non-empty Logos Storage CID is recorded (fetch is a separate step).
    Available(String),
    /// No usable CID was recorded; metadata still renders without it.
    Unavailable,
}

/// Filter decoded entries by tag (OR semantics), author, and verification flag.
pub fn search_entries<'a, I>(
    entries: I,
    tags: &[&str],
    author: Option<&[u8; 32]>,
    verified_only: bool,
) -> Vec<&'a RegistryEntry>
where
    I: IntoIterator<Item = &'a RegistryEntry>,
{
    search_entries_with_name(entries, tags, author, verified_only, None)
}

/// Filter decoded entries by name, tag (OR semantics), author and verification.
///
/// `name` is a case-insensitive substring match, so searching for "registry"
/// finds "LP-0023 Registry v5". Passing `None` preserves the behaviour of
/// [`search_entries`].
pub fn search_entries_with_name<'a, I>(
    entries: I,
    tags: &[&str],
    author: Option<&[u8; 32]>,
    verified_only: bool,
    name: Option<&str>,
) -> Vec<&'a RegistryEntry>
where
    I: IntoIterator<Item = &'a RegistryEntry>,
{
    entries
        .into_iter()
        .filter(|e| name.is_none_or(|n| e.matches_name(n)))
        .filter(|e| tags.is_empty() || e.matches_any_tag(tags.iter().copied()))
        .filter(|e| author.is_none_or(|a| e.author == *a))
        .filter(|e| !verified_only || e.is_independently_verified())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, tags: &str, author: u8, verified_by: Option<u8>) -> RegistryEntry {
        RegistryEntry {
            program_id: [7; 32],
            name: name.into(),
            version: "0.1.0".into(),
            author: [author; 32],
            description: "test".into(),
            tags: tags.into(),
            idl_cid: "idl".into(),
            code_cid: "src".into(),
            commit: "abc123".into(),
            build_config: "guest=local".into(),
            timestamp: 1_700_000_000,
            deployer_claimed: true,
            source_verified: verified_by.is_some(),
            verifier: verified_by.map(|v| [v; 32]),
            verification_method: if verified_by.is_some() {
                "reproducible-rebuild:program-id".into()
            } else {
                String::new()
            },
            evidence_cid: if verified_by.is_some() {
                "evidence".into()
            } else {
                String::new()
            },
        }
    }

    #[test]
    fn borsh_roundtrip_matches_account_encoding() {
        let original = entry("roundtrip", "a,b", 1, Some(2));
        let bytes = original.to_account_data().unwrap();
        let decoded = RegistryEntry::from_account_data(&bytes).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn tag_match_is_case_insensitive() {
        let e = entry("tagged", "Registry, SPEL,lez", 1, None);
        assert!(e.matches_tag("spel"));
        assert!(e.matches_tag("REGISTRY"));
        assert!(e.matches_any_tag(["missing", "Lez"]));
        assert!(!e.matches_any_tag(["missing", "nope"]));
    }

    #[test]
    fn index_roundtrip() {
        let index = RegistryIndex {
            program_ids: vec![[9; 32], [8; 32]],
        };
        let bytes = index.to_account_data().unwrap();
        assert_eq!(RegistryIndex::from_account_data(&bytes).unwrap(), index);
    }

    #[test]
    fn search_filters_by_tag_author_and_verification() {
        let entries = vec![
            entry("one", "registry,spel", 1, Some(2)),
            entry("two", "registry", 1, None),
            entry("three", "wallet", 3, Some(4)),
        ];
        let by_tag = search_entries(&entries, &["spel"], None, false);
        assert_eq!(by_tag.len(), 1);
        assert_eq!(by_tag[0].name, "one");

        let by_author = search_entries(&entries, &[], Some(&[1; 32]), false);
        assert_eq!(by_author.len(), 2);

        let verified = search_entries(&entries, &[], None, true);
        assert_eq!(verified.len(), 2);
        assert!(verified.iter().all(|e| e.is_independently_verified()));

        let verified_registry = search_entries(&entries, &["registry"], None, true);
        assert_eq!(verified_registry.len(), 1);
        assert_eq!(verified_registry[0].name, "one");
    }

    #[test]
    fn name_match_is_case_insensitive_substring() {
        let e = entry("LP-0023 Registry v5", "registry", 1, None);
        assert!(e.matches_name("registry"));
        assert!(e.matches_name("REGISTRY"));
        assert!(e.matches_name("0023"));
        assert!(e.matches_name("  lp-0023  "));
        assert!(!e.matches_name("wallet"));
    }

    #[test]
    fn search_filters_by_name() {
        let entries = vec![
            entry("LP-0023 Registry v5", "registry", 1, Some(2)),
            entry("Wallet Index", "wallet", 3, None),
            entry("registry-browser", "tool", 4, None),
        ];

        let by_name = search_entries_with_name(&entries, &[], None, false, Some("registry"));
        assert_eq!(by_name.len(), 2);

        // Name and tag compose (AND across filter kinds).
        let name_and_tag =
            search_entries_with_name(&entries, &["registry"], None, false, Some("lp-0023"));
        assert_eq!(name_and_tag.len(), 1);
        assert_eq!(name_and_tag[0].name, "LP-0023 Registry v5");

        // Existing callers keep their behaviour when no name is supplied.
        assert_eq!(search_entries(&entries, &[], None, false).len(), 3);
    }

    #[test]
    fn idl_availability_degrades_without_a_usable_cid() {
        let mut e = entry("with-idl", "x", 1, None);
        assert_eq!(
            e.idl_availability(),
            IdlAvailability::Available("idl".into())
        );

        for placeholder in ["", "   ", "none", "NULL", "Placeholder"] {
            e.idl_cid = placeholder.into();
            assert_eq!(e.idl_availability(), IdlAvailability::Unavailable);
        }
    }
}

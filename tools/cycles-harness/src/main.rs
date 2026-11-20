//! proclaim-specific guest cycle-count harness (LP-0023).
//!
//! WHY THIS EXISTS
//! ---------------
//! LEZ `cycle_bench` only benchmarks its built-in guests and cannot load a
//! deployed program ID. This harness executes the exact R0BF artifact whose
//! image ID is the deployed proclaim program ID, with synthetic fixtures that
//! mirror the six `#[instruction]` functions in
//! `methods/guest/src/bin/registry.rs`.
//!
//! AUTHORITATIVE API FACTS VERIFIED FROM LOCAL SOURCE (not guessed):
//! * risc0-zkvm 3.0.5 `default_executor().execute(env, elf)` returns
//!   `SessionInfo`; `info.cycles()` sums `segments[*].cycles` (user cycles, no
//!   continuation/po2 overhead) and `info.segments.len()` is the segment count.
//!   (risc0-zkvm-3.0.5/src/host/api/mod.rs, host/client/prove/mod.rs)
//! * `ExecutorImpl::from_elf` calls `ProgramBinary::decode(elf)`, so the bytes
//!   passed to `execute()` MUST be the R0BF-wrapped `.bin`, not a raw ELF.
//!   (risc0-zkvm-3.0.5/src/host/server/exec/executor.rs::from_elf)
//! * `risc0_zkvm::compute_image_id(&r0bf_bytes)` gives the image ID; the hex
//!   program ID is `id.iter().flat_map(u32::to_le_bytes)` (verified against
//!   spel-cli/src/inspect.rs).
//! * The guest reads four env writes in order (lee_core `read_lee_inputs`):
//!     ProgramId, Option<ProgramId>, Vec<AccountWithMetadata>, Vec<u32>.
//! * `instruction_words` are `risc0_zkvm::serde::to_vec(&Instruction)` where
//!   the SPEL-generated enum has one variant per `#[instruction]` in source
//!   order; a unit variant writes only its u32 index, a struct variant writes
//!   its u32 index followed by fields. (spel-framework-macros + spel-cli
//!   `serialize_to_risc0`; risc0-zkvm serde Serializer.)

use std::{path::PathBuf, time::Instant};

use anyhow::{Context, Result, bail};
use borsh::{BorshDeserialize, BorshSerialize};
use nssa_core::{
    account::{Account, AccountId, AccountWithMetadata, Data},
    program::{PdaSeed, ProgramId},
};
use serde::Serialize;
use sha2::{Digest as _, Sha256};

const EXPECTED_PROGRAM_ID: &str =
    "4df7d5396b68a16aae6d0f768ef447a0055eb43866827154f71139e486265144";
const EXPECTED_ELF_SHA256: &str =
    "88716a6ba8c67e9623f3b2d5acf1d9c758ce071dbce391f1d9084628d72506c5";
const DEFAULT_ELF: &str = "/home/xps/logos-work/registry/methods/guest/target/riscv32im-risc0-zkvm-elf/docker/registry.bin";
const GUEST_REPO: &str = "/home/xps/logos-work/registry";
const INDEX_SEED: &str = "registry_index";

/// Host-side implementation required by the task's
/// `RUSTFLAGS='--cfg getrandom_backend="custom"'`. The same symbol name is
/// required by getrandom 0.4.x when that cfg is active; without it, host build
/// scripts fail to link with `undefined symbol: __getrandom_v03_custom`.
/// We read /dev/urandom directly (no real guest entropy is involved; this is
/// only for the local host harness process).
#[no_mangle]
unsafe extern "Rust" fn __getrandom_v03_custom(
    dest: *mut u8,
    len: usize,
) -> std::result::Result<(), getrandom::Error> {
    if len == 0 {
        return Ok(());
    }
    let slice = unsafe { core::slice::from_raw_parts_mut(dest, len) };
    match std::fs::File::open("/dev/urandom").and_then(|mut f| {
        use std::io::Read as _;
        f.read_exact(slice)
    }) {
        Ok(()) => Ok(()),
        Err(_) => Err(getrandom::Error::new_custom(1)),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Borsh layouts copied byte-for-byte from `registry_core` / the guest source.
// These are FIXTURES (synthetic test data), not real on-chain account state.
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
struct RegistryEntry {
    program_id: [u8; 32],
    name: String,
    version: String,
    author: [u8; 32],
    description: String,
    tags: String,
    idl_cid: String,
    code_cid: String,
    commit: String,
    build_config: String,
    timestamp: u64,
    deployer_claimed: bool,
    source_verified: bool,
    verifier: Option<[u8; 32]>,
    verification_method: String,
    evidence_cid: String,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
struct RegistryIndex {
    program_ids: Vec<[u8; 32]>,
}

/// Mirror of the SPEL-generated `Instruction` enum. Variant order MUST match
/// the declaration order in `methods/guest/src/bin/registry.rs`.
#[derive(Debug, Clone, Serialize)]
enum Instruction {
    InitIndex,
    Register {
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
    },
    Update {
        key: [u8; 32],
        version: String,
        description: String,
        tags: String,
        idl_cid: String,
        code_cid: String,
        commit: String,
        build_config: String,
    },
    MarkVerified {
        key: [u8; 32],
        method: String,
        evidence_cid: String,
    },
    Lookup {
        key: [u8; 32],
    },
    Search,
}

// ─────────────────────────────────────────────────────────────────────────────
// Fixture helpers. All IDs/strings below are SYNTHETIC and exist only to make
// the guest's account validation (signer/init/PDA) and Borsh decodes succeed.
// ─────────────────────────────────────────────────────────────────────────────

fn seed_from_str(s: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[..s.len()].copy_from_slice(s.as_bytes());
    out
}

fn pda(program_id: &ProgramId, seed: [u8; 32]) -> AccountId {
    AccountId::for_public_pda(program_id, &PdaSeed::new(seed))
}

fn account(owner: ProgramId, bytes: Vec<u8>, authorized: bool, id: AccountId) -> AccountWithMetadata {
    AccountWithMetadata {
        account: Account {
            program_owner: owner,
            balance: 0,
            data: Data::try_from(bytes).expect("fixture data fits in account data"),
            nonce: 0_u128.into(),
        },
        is_authorized: authorized,
        account_id: id,
    }
}

fn default_account(id: AccountId, authorized: bool) -> AccountWithMetadata {
    AccountWithMetadata {
        account: Account::default(),
        is_authorized: authorized,
        account_id: id,
    }
}

fn entry_data(author: [u8; 32]) -> Vec<u8> {
    borsh::to_vec(&RegistryEntry {
        program_id: [7u8; 32],
        name: "LP-0023 fixture".into(),
        version: "0.1.0".into(),
        author,
        description: "synthetic fixture for cycle measurement".into(),
        tags: "registry,spel".into(),
        idl_cid: "bafy-fixture-idl".into(),
        code_cid: "bafy-fixture-source".into(),
        commit: "0000000".into(),
        build_config: "guest=fixture".into(),
        timestamp: 1_700_000_000,
        deployer_claimed: true,
        source_verified: false,
        verifier: None,
        verification_method: String::new(),
        evidence_cid: String::new(),
    })
    .expect("borsh serialize RegistryEntry fixture")
}

fn index_data(n: usize) -> Vec<u8> {
    // Deliberately avoid the fixture `key` ([7u8; 32]) so `register` exercises
    // the append-to-index path rather than the contains() short-circuit.
    let program_ids = (0..n).map(|i| [i as u8 + 10; 32]).collect();
    borsh::to_vec(&RegistryIndex { program_ids }).expect("borsh serialize RegistryIndex fixture")
}

// ─────────────────────────────────────────────────────────────────────────────
// Benchmark case
// ─────────────────────────────────────────────────────────────────────────────

struct Case {
    instruction: &'static str,
    pre_states: Vec<AccountWithMetadata>,
    instruction_words: Vec<u32>,
    fixture_note: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct CycleStats {
    /// Best (minimum) observed user-cycle count.
    min: u64,
    median: u64,
    p95: u64,
    max: u64,
    mean: f64,
    /// Population standard deviation.
    stdev: f64,
}

#[derive(Debug, Clone, Serialize)]
struct CaseResult {
    instruction: &'static str,
    cycles: CycleStats,
    segments_min: usize,
    segments_median: usize,
    segments_max: usize,
    iterations: usize,
    /// Number of risc0 words used to encode this instruction fixture.
    instruction_words_len: usize,
    fixture_note: &'static str,
    /// True only for the warm-up run that is discarded; always 1 here.
    warmup_discarded: usize,
}

fn percentile_nearest_rank(sorted: &[u64], p: f64) -> u64 {
    let rank = (p * sorted.len() as f64).ceil() as usize;
    sorted[rank.clamp(1, sorted.len()) - 1]
}

fn stats(samples: &[u64]) -> CycleStats {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let mean = samples.iter().sum::<u64>() as f64 / samples.len() as f64;
    let variance = samples
        .iter()
        .map(|v| {
            let d = *v as f64 - mean;
            d * d
        })
        .sum::<f64>()
        / samples.len() as f64;
    CycleStats {
        min: sorted[0],
        median: sorted[sorted.len() / 2],
        p95: percentile_nearest_rank(&sorted, 0.95),
        max: sorted[sorted.len() - 1],
        mean,
        stdev: variance.sqrt(),
    }
}

fn median_usize(mut v: Vec<usize>) -> usize {
    v.sort_unstable();
    v[v.len() / 2]
}

fn build_cases(program_id: &ProgramId, index_len: usize) -> Vec<Case> {
    // FIXTURE IDs: arbitrary 32-byte values. No real account or key material.
    let author = [2u8; 32];
    let verifier = [3u8; 32];
    let authority = [4u8; 32];
    let key = [7u8; 32]; // also the entry PDA seed and RegistryEntry.program_id

    let index_id = pda(program_id, seed_from_str(INDEX_SEED));
    let entry_id = pda(program_id, key);

    let mut cases = Vec::new();

    // 1. init_index: index PDA must be Account::default() (`init` constraint),
    //    authority must be a signer.
    cases.push(Case {
        instruction: "init_index",
        pre_states: vec![
            default_account(index_id, false),
            default_account(AccountId::new(authority), true),
        ],
        instruction_words: risc0_zkvm::serde::to_vec(&Instruction::InitIndex)
            .expect("serialize init_index"),
        fixture_note: "synthetic: empty index PDA + signer authority",
    });

    // 2. register: entry PDA is `init` (default), index PDA holds an N-entry
    //    Borsh index, author is a signer. Cost grows with N.
    cases.push(Case {
        instruction: "register",
        pre_states: vec![
            default_account(entry_id, false),
            account(*program_id, index_data(index_len), false, index_id),
            default_account(AccountId::new(author), true),
        ],
        instruction_words: risc0_zkvm::serde::to_vec(&Instruction::Register {
            key,
            name: "LP-0023 fixture".into(),
            version: "0.1.0".into(),
            description: "synthetic fixture for cycle measurement".into(),
            tags: "registry,spel".into(),
            idl_cid: "bafy-fixture-idl".into(),
            code_cid: "bafy-fixture-source".into(),
            commit: "0000000".into(),
            build_config: "guest=fixture".into(),
            timestamp: 1_700_000_000,
            deployer_claimed: true,
        })
        .expect("serialize register"),
        fixture_note: "synthetic: init entry PDA + N-entry index + signer author",
    });

    // 3. update: existing entry owned by `author`, author is signer.
    cases.push(Case {
        instruction: "update",
        pre_states: vec![
            account(*program_id, entry_data(author), false, entry_id),
            default_account(AccountId::new(author), true),
        ],
        instruction_words: risc0_zkvm::serde::to_vec(&Instruction::Update {
            key,
            version: "0.2.0".into(),
            description: "synthetic updated description".into(),
            tags: "registry,spel,updated".into(),
            idl_cid: "bafy-fixture-idl".into(),
            code_cid: "bafy-fixture-source-v2".into(),
            commit: "1111111".into(),
            build_config: "guest=fixture-v2".into(),
        })
        .expect("serialize update"),
        fixture_note: "synthetic: existing entry + signer author; source fields change",
    });

    // 4. mark_verified: verifier is a DIFFERENT signer from the author.
    cases.push(Case {
        instruction: "mark_verified",
        pre_states: vec![
            account(*program_id, entry_data(author), false, entry_id),
            default_account(AccountId::new(verifier), true),
        ],
        instruction_words: risc0_zkvm::serde::to_vec(&Instruction::MarkVerified {
            key,
            method: "reproducible-rebuild:program-id".into(),
            evidence_cid: "bafy-fixture-evidence".into(),
        })
        .expect("serialize mark_verified"),
        fixture_note: "synthetic: existing entry + independent signer verifier",
    });

    // 5. lookup: read-only, one entry PDA, no signer.
    cases.push(Case {
        instruction: "lookup",
        pre_states: vec![account(*program_id, entry_data(author), false, entry_id)],
        instruction_words: risc0_zkvm::serde::to_vec(&Instruction::Lookup { key })
            .expect("serialize lookup"),
        fixture_note: "synthetic: existing entry PDA, read-only",
    });

    // 6. search: read-only, index PDA only.
    cases.push(Case {
        instruction: "search",
        pre_states: vec![account(*program_id, index_data(index_len), false, index_id)],
        instruction_words: risc0_zkvm::serde::to_vec(&Instruction::Search)
            .expect("serialize search"),
        fixture_note: "synthetic: N-entry index PDA, read-only",
    });

    cases
}

fn main() -> Result<()> {
    // Minimal argument parsing; no extra CLI dependency needed.
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut elf_path = PathBuf::from(DEFAULT_ELF);
    let mut iterations = 20usize;
    let mut index_len = 8usize;
    let mut output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cycles-report.json");
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--elf" => {
                i += 1;
                elf_path = PathBuf::from(args.get(i).context("--elf needs a path")?);
            }
            "--iterations" => {
                i += 1;
                iterations = args
                    .get(i)
                    .context("--iterations needs a number")?
                    .parse()?;
            }
            "--index-len" => {
                i += 1;
                index_len = args
                    .get(i)
                    .context("--index-len needs a number")?
                    .parse()?;
            }
            "--output" => {
                i += 1;
                output = PathBuf::from(args.get(i).context("--output needs a path")?);
            }
            other => bail!("unknown argument: {other}"),
        }
        i += 1;
    }
    if iterations < 2 {
        bail!("--iterations must be >= 2 (1 warm-up + at least 1 sample)");
    }

    // Load the R0BF-wrapped artifact. The executor decodes ProgramBinary, so a
    // raw ELF would fail here; the wrapper is required by risc0 3.x.
    let elf_bytes = std::fs::read(&elf_path)
        .with_context(|| format!("reading R0BF artifact {}", elf_path.display()))?;
    let wrapped_sha256 = hex(Sha256::digest(&elf_bytes).as_slice());
    if wrapped_sha256 != EXPECTED_ELF_SHA256 {
        bail!(
            "ELF SHA-256 mismatch: computed {wrapped_sha256}, expected {EXPECTED_ELF_SHA256}. \
             Refusing to report cycles for an unexpected artifact."
        );
    }
    eprintln!("ELF SHA-256 asserted: {wrapped_sha256}");

    // Assert the computed image ID equals the deployed program ID.
    let image_id: ProgramId = risc0_zkvm::compute_image_id(&elf_bytes)
        .context("computing image ID from R0BF artifact")?
        .into();
    let computed_id = hex(&image_id.iter().flat_map(|w| w.to_le_bytes()).collect::<Vec<u8>>());
    if computed_id != EXPECTED_PROGRAM_ID {
        bail!(
            "program ID mismatch: computed {computed_id}, expected {EXPECTED_PROGRAM_ID}. \
             The measured ELF is NOT the deployed artifact; refusing to report cycles."
        );
    }
    eprintln!("program ID asserted: {computed_id}");

    // Ensure the in-process executor is selected. (With the `prove` feature,
    // this is already the default; setting it explicitly removes ambiguity.)
    if std::env::var("RISC0_EXECUTOR").unwrap_or_default().is_empty() {
        std::env::set_var("RISC0_EXECUTOR", "local");
    }

    let cases = build_cases(&image_id, index_len);
    let executor = risc0_zkvm::default_executor();
    let mut results = Vec::new();

    for case in &cases {
        let mut cycles_samples = Vec::with_capacity(iterations);
        let mut segment_samples = Vec::with_capacity(iterations);

        // One warm-up pass is discarded, matching LEZ cycle_bench::sample().
        for iter in 0..=iterations {
            let mut env_builder = risc0_zkvm::ExecutorEnv::builder();
            env_builder
                .write(&image_id)?
                .write(&Option::<ProgramId>::None)?
                .write(&case.pre_states)?
                .write(&case.instruction_words)?;
            let env = env_builder.build()?;

            let started = Instant::now();
            let info = executor
                .execute(env, &elf_bytes)
                .with_context(|| format!("executing {} fixture", case.instruction))?;
            let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
            if iter == 0 {
                eprintln!("  warmup {}: {:.1} ms", case.instruction, elapsed_ms);
                continue;
            }
            cycles_samples.push(info.cycles());
            segment_samples.push(info.segments.len());
        }

        results.push(CaseResult {
            instruction: case.instruction,
            cycles: stats(&cycles_samples),
            segments_min: *segment_samples.iter().min().expect("at least one sample"),
            segments_median: median_usize(segment_samples.clone()),
            segments_max: *segment_samples.iter().max().expect("at least one sample"),
            iterations,
            instruction_words_len: case.instruction_words.len(),
            fixture_note: case.fixture_note,
            warmup_discarded: 1,
        });
    }

    // Markdown table on stdout, as required by the closeout plan.
    println!(
        "| Instruction | user cycles (best / median / p95 / mean±stdev) | segments | words | iters | N | Fixture |"
    );
    println!("| --- | --- | --- | --- | --- | --- | --- |");
    for r in &results {
        println!(
            "| `{}` | {} / {} / {} / {:.1}±{:.1} | {} (min {} max {}) | {} | {} | {} | {} |",
            r.instruction,
            r.cycles.min,
            r.cycles.median,
            r.cycles.p95,
            r.cycles.mean,
            r.cycles.stdev,
            r.segments_median,
            r.segments_min,
            r.segments_max,
            r.instruction_words_len,
            r.iterations,
            index_len,
            r.fixture_note
        );
    }

    let report = serde_json::json!({
        "schema": "proclaim-cycle-harness/1",
        "generated_at_utc": chrono_like_utc(),
        "program_id": {
            "expected": EXPECTED_PROGRAM_ID,
            "computed": computed_id,
            "match": true,
        },
        "elf": {
            "path": elf_path.to_string_lossy(),
            "sha256": wrapped_sha256,
            "size_bytes": elf_bytes.len(),
            "format": "R0BF ProgramBinary (risc0-binfmt 3.0.5)",
        },
        "method": {
            "executor": "risc0_zkvm::default_executor (LocalProver in-process; prove feature enabled)",
            "call": "default_executor().execute(env, &r0bf_bytes)",
            "warmup_discarded": 1,
            "iterations": iterations,
            "p95_definition": "nearest-rank: sorted[ceil(0.95*n)-1]",
            "median_definition": "upper median: sorted[n/2] for 0-based n",
            "cycles_source": "SessionInfo::cycles() (sum of segment user cycles, no po2 padding)",
            "segments_source": "SessionInfo::segments.len()",
            "total_cycles_po2": null,
            "total_cycles_po2_note": "Not measured: this execution-only run does not invoke a Prover. No po2/proving number is fabricated.",
            "stdev_definition": "population standard deviation over sampled SessionInfo::cycles() values",
            "best_definition": "minimum of sampled SessionInfo::cycles() values",
            "chained_calls": "none: all six guest handlers return SpelOutput::execute(post_states, vec![])",
            "index_length_n": index_len,
            "fixtures_are_synthetic": true,
            "fixtures_note": "All account IDs, author/verifier keys, strings and CIDs are synthetic fixtures; no real on-chain state or private keys are used.",
        },
        "toolchain": {
            "risc0_zkvm": "=3.0.5 (default-features=false, features=std,prove)",
            "risc0_binfmt_locked": "3.0.5; transitive risc0-binfmt 3.0.6 was compiled by cargo",
            "lee_core_git_tag": "v0.2.4",
            "lee_core_git_rev": "47eba256479f6f785acbd138834340703cd03401",
            "spel_framework_git_tag": "v0.7.0 (guest build pin)",
            "rustc": command_line("rustc", &["-Vv"]),
            "guest_repo": GUEST_REPO,
            "guest_repo_commit": command_line("git", &["-C", GUEST_REPO, "rev-parse", "HEAD"]),
        },
        "environment": {
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "kernel": command_line("uname", &["-a"]),
            "cpu_model": cpu_model(),
            "host_profile": "dev (host debug build; guest cycle counts are VM-deterministic)",
        },
        "results": results,
    });

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&output, serde_json::to_string_pretty(&report)?)?;
    println!("\nJSON written to {}", output.display());
    Ok(())
}

/// First non-empty line of `command` output, or `unavailable`.
fn command_line(command: &str, args: &[&str]) -> String {
    std::process::Command::new(command)
        .args(args)
        .output()
        .ok()
        .and_then(|out| {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(str::trim)
                .find(|line| !line.is_empty())
                .map(ToOwned::to_owned)
        })
        .unwrap_or_else(|| "unavailable".into())
}

fn cpu_model() -> String {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|line| line.starts_with("model name"))
                .and_then(|line| line.split_once(':'))
                .map(|(_, value)| value.trim().to_owned())
        })
        .unwrap_or_else(|| "unavailable".into())
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        write!(&mut s, "{b:02x}").expect("write to String cannot fail");
    }
    s
}

/// Small UTC timestamp without pulling in chrono.
fn chrono_like_utc() -> String {
    // UNIX seconds -> civil date (Howard Hinnant's civil_from_days algorithm).
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days = secs.div_euclid(86_400);
    let sod = secs.rem_euclid(86_400);
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let mut y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    if m <= 2 {
        y += 1;
    }
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        sod / 3600,
        (sod % 3600) / 60,
        sod % 60
    )
}

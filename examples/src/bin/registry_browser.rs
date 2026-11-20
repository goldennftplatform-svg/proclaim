//! Host-side browser for the LP-0023 LEZ Program Registry.
//!
//! Enumerates the on-chain global index, derives each entry PDA, fetches and
//! Borsh-decodes the entries, then filters by name / tag / author / verification.
//! Reads the sequencer connection from `LEE_WALLET_HOME_DIR`
//! (wallet_config.json + storage.json).
//!
//! Dual-licensed: MIT OR Apache-2.0.

use std::env;
use std::process;

use nssa::AccountId;
use registry_api::{to_base58, DEFAULT_PROGRAM_ID_HEX};
use registry_core::{RegistryEntry, RegistryIndex};
use wallet::WalletCore;

#[derive(Default, Debug)]
struct Args {
    program_id_hex: String,
    name: Option<String>,
    tags: Vec<String>,
    authors: Vec<[u8; 32]>,
    verified_only: bool,
    json: bool,
    index_only: bool,
    limit: Option<usize>,
}

fn usage() -> ! {
    eprintln!(
        "registry_browser — enumerate & filter on-chain registry entries\n\
         \n\
         USAGE:\n\
         \x20 registry_browser [OPTIONS]\n\
         \n\
         OPTIONS:\n\
         \x20 --program-id <hex>   registry program id (default: v5 deployed id)\n\
         \x20 --name <substring>  filter by name (case-insensitive substring)\n\
         \x20 --tag <tag>          filter by tag (repeatable, OR semantics)\n\
         \x20 --author <key>       filter by author account (base58 or 64-hex, repeatable)\n\
         \x20 --verified           only independently verified entries\n\
         \x20 --json               emit full JSON (default: table)\n\
         \x20 --index-only         just dump the global index (no entry fetches)\n\
         \x20 --limit <n>          cap number of entries displayed\n\
         \x20 -h, --help           show this help\n\
         \n\
         ENV:\n\
         \x20 LEE_WALLET_HOME_DIR  path to wallet home (default: .scaffold/wallet)\n"
    );
    process::exit(0);
}

fn parse_args() -> Args {
    let argv: Vec<String> = env::args().skip(1).collect();
    let mut a = Args {
        program_id_hex: DEFAULT_PROGRAM_ID_HEX.to_string(),
        ..Default::default()
    };
    let mut i = 0;
    while i < argv.len() {
        match argv[i].as_str() {
            "-h" | "--help" => usage(),
            "--program-id" => {
                i += 1;
                a.program_id_hex = argv.get(i).cloned().unwrap_or_else(|| die("--program-id needs a value"));
            }
            "--name" => {
                i += 1;
                a.name = Some(argv.get(i).cloned().unwrap_or_else(|| die("--name needs a value")));
            }
            "--tag" => {
                i += 1;
                a.tags.push(argv.get(i).cloned().unwrap_or_else(|| die("--tag needs a value")));
            }
            "--author" => {
                i += 1;
                let s = argv.get(i).cloned().unwrap_or_else(|| die("--author needs a value"));
                a.authors.push(parse_key32(&s));
            }
            "--verified" => a.verified_only = true,
            "--json" => a.json = true,
            "--index-only" => a.index_only = true,
            "--limit" => {
                i += 1;
                let s = argv.get(i).cloned().unwrap_or_else(|| die("--limit needs a value"));
                a.limit = Some(s.parse().unwrap_or_else(|_| die("--limit must be a number")));
            }
            other => die(&format!("unknown argument: {}", other)),
        }
        i += 1;
    }
    a
}

fn die(msg: &str) -> ! {
    eprintln!("error: {}", msg);
    process::exit(1);
}

/// Decode a 32-byte key from 64-char hex or base58 (shared client helper).
fn parse_key32(s: &str) -> [u8; 32] {
    registry_api::parse_key32(s).unwrap_or_else(|e| die(&e.to_string()))
}

fn short_b58(key: &[u8; 32]) -> String {
    let s = to_base58(key);
    if s.len() > 12 {
        format!("{}…{}", &s[..6], &s[s.len() - 4..])
    } else {
        s
    }
}

fn iso_time(ts: u64) -> String {
    // Minimal UTC conversion (days since epoch -> YYYY-MM-DD) without chrono.
    let secs = ts as i64;
    let days = secs.div_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    format!("{:04}-{:02}-{:02}", y, m, d)
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    (y + if m <= 2 { 1 } else { 0 }, m, d)
}

async fn fetch_data(wc: &WalletCore, id: AccountId) -> Option<Vec<u8>> {
    match registry_api::fetch_account_data(wc, id).await {
        Ok(data) => Some(data),
        Err(e) => {
            eprintln!("  warn: {}", e);
            None
        }
    }
}

#[tokio::main]
async fn main() {
    let args = parse_args();
    let program_id = registry_api::program_id_from_hex(&args.program_id_hex)
        .unwrap_or_else(|e| die(&e.to_string()));

    eprintln!("registry program id : 0x{}", args.program_id_hex.trim_start_matches("0x"));
    let idx_id = registry_api::index_pda(&program_id).unwrap_or_else(|e| die(&e.to_string()));
    eprintln!("index PDA           : {}", idx_id);

    let wallet_home = env::var("LEE_WALLET_HOME_DIR").unwrap_or_else(|_| {
        env::current_dir()
            .unwrap_or_default()
            .join(".scaffold/wallet")
            .to_string_lossy()
            .to_string()
    });
    eprintln!("wallet home         : {}", wallet_home);

    let wc = WalletCore::from_env().await.unwrap_or_else(|e| {
        eprintln!("failed to init wallet: {}", e);
        eprintln!("set LEE_WALLET_HOME_DIR to the registry wallet home");
        process::exit(1);
    });

    let index_bytes = fetch_data(&wc, idx_id).await.unwrap_or_else(|| {
        eprintln!("index account is empty — has `init-index` been run?");
        process::exit(1);
    });
    let index = RegistryIndex::from_account_data(&index_bytes).unwrap_or_else(|e| {
        eprintln!("failed to decode RegistryIndex: {}", e);
        process::exit(1);
    });
    eprintln!("registered programs : {}", index.program_ids.len());

    if args.json && args.index_only {
        let ids: Vec<String> = index.program_ids.iter().map(|k| to_base58(k)).collect();
        println!("{}", serde_json::to_string_pretty(&ids).unwrap());
        return;
    }
    if args.index_only {
        println!("{:<52}  {}", "PROGRAM ID (base58)", "HEX");
        for k in &index.program_ids {
            println!("{:<52}  0x{}", to_base58(k), k.iter().map(|b| format!("{:02x}", b)).collect::<String>());
        }
        return;
    }

    let mut entries: Vec<RegistryEntry> = Vec::new();
    for (n, key) in index.program_ids.iter().enumerate() {
        let id = registry_api::entry_pda(&program_id, key);
        eprintln!("[{}/{}] fetching {}", n + 1, index.program_ids.len(), id);
        if let Some(data) = fetch_data(&wc, id).await {
            match RegistryEntry::from_account_data(&data) {
                Ok(e) => entries.push(e),
                Err(e) => eprintln!("  warn: failed to decode entry {}: {}", id, e),
            }
        }
    }

    let author_refs: Vec<&[u8; 32]> = args.authors.iter().collect();
    let matched = registry_api::filter_entries(
        &entries,
        &args.tags,
        None,
        args.verified_only,
        args.name.as_deref(),
    );
    let matched: Vec<&RegistryEntry> = if author_refs.is_empty() {
        matched
    } else {
        matched
            .into_iter()
            .filter(|e| author_refs.iter().any(|a| e.author == **a))
            .collect()
    };
    let matched: Vec<&RegistryEntry> = match args.limit {
        Some(n) => matched.into_iter().take(n).collect(),
        None => matched,
    };

    eprintln!("matched entries     : {} (of {} decoded)", matched.len(), entries.len());

    if args.json {
        let vals: Vec<serde_json::Value> = matched
            .iter()
            .map(|e| serde_json::to_value(e).unwrap())
            .collect();
        println!("{}", serde_json::to_string_pretty(&vals).unwrap());
        return;
    }

    println!(
        "{:<20} {:<8} {:<12} {:<12} {:<22} {:<4} {}",
        "NAME", "VER", "PROGRAM", "AUTHOR", "TAGS", "VER", "DATE"
    );
    println!("{}", "-".repeat(110));
    for e in &matched {
        println!(
            "{:<20} {:<8} {:<12} {:<12} {:<22} {:<4} {}",
            truncate(&e.name, 20),
            truncate(&e.version, 8),
            short_b58(&e.program_id),
            short_b58(&e.author),
            truncate(&e.tags, 22),
            if e.is_independently_verified() { "yes" } else { "-" },
            iso_time(e.timestamp),
        );
    }
    if matched.is_empty() {
        println!("(no entries matched the current filters)");
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max - 1).collect::<String>())
    }
}

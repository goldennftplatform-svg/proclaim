#!/bin/bash
# LP-0023 independent reproducible-build verifier.
#
# Rebuilds the guest from source in a clean target directory, wraps the raw ELF
# in R0BF format, derives the program ID, and compares it with the program ID
# recorded on-chain. On a match it prints the exact `spel mark-verified`
# invocation an independent verifier should submit.
#
# Usage:
#   scripts/verify-source.sh <REGISTERED_PROGRAM_ID_HEX> [SOURCE_DIR] [EVIDENCE_CID]
#
# Example:
#   scripts/verify-source.sh \
#     148657614d74b2a7a7cb764804a41f915ae4624128d808b8c2626fec27777dd5 \
#     /home/xps/logos-work/registry QmRebuildEvidenceV4

set -euo pipefail

if [ $# -lt 1 ]; then
  echo "Usage: $0 <REGISTERED_PROGRAM_ID_HEX> [SOURCE_DIR] [EVIDENCE_CID]" >&2
  exit 2
fi

EXPECTED=$(printf '%s' "$1" | tr 'A-Z' 'a-z')
SOURCE_DIR=${2:-"$(cd "$(dirname "$0")/.." && pwd)"}
EVIDENCE_CID=${3:-QmRebuildEvidence}
MAKE_BIN=${MAKE_BIN:-/home/xps/logos-work/make-bin/target/release/make-bin}

. "$HOME/.cargo/env"
export PATH="$HOME/.risc0/bin:$PATH"
export CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-2}
export RUSTFLAGS='--cfg getrandom_backend="custom"'

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
export CARGO_TARGET_DIR="$WORK/target"

echo "[1/4] Clean rebuild from: $SOURCE_DIR"
cargo +risc0 build --release \
  --manifest-path "$SOURCE_DIR/methods/guest/Cargo.toml" \
  --target riscv32im-risc0-zkvm-elf

ELF="$CARGO_TARGET_DIR/riscv32im-risc0-zkvm-elf/release/registry"
if [ ! -f "$ELF" ]; then
  echo "ERROR: rebuilt ELF not found at $ELF" >&2
  exit 1
fi

echo "[2/4] Wrapping ELF in R0BF program-binary format"
"$MAKE_BIN" "$ELF" "$WORK/registry.bin"

echo "[3/4] Deriving rebuilt program ID"
REBUILT=$(lgs spel -- program-id "$WORK/registry.bin" --format hex 2>/dev/null | tail -1)
echo "  registered: $EXPECTED"
echo "  rebuilt:    $REBUILT"

echo "[4/4] Comparing"
if [ "$REBUILT" != "$EXPECTED" ]; then
  echo "❌ MISMATCH — rebuilt bytecode does not match the on-chain record."
  echo "   Check toolchain pins: risc0=3.0.5, V1COMPAT_ELF=2.2.3, RUSTFLAGS above."
  exit 1
fi

cat > "$WORK/attestation.json" <<EOF
{
  "scheme": "lp0023-reproducible-rebuild-v1",
  "program_id": "$REBUILT",
  "method": "reproducible-rebuild:program-id",
  "evidence_cid": "$EVIDENCE_CID",
  "rustflags": "--cfg getrandom_backend=\"custom\"",
  "risc0": "3.0.5",
  "v1compat": "2.2.3"
}
EOF

echo "✅ MATCH — byte-for-byte reproduction confirmed."
echo
echo "Submit from an account distinct from the registering author:"
echo
echo "  lgs spel -- mark-verified \\"
echo "    --key $REBUILT \\"
echo "    --method 'reproducible-rebuild:program-id' \\"
echo "    --evidence-cid '$EVIDENCE_CID' \\"
echo "    --verifier Public/<YOUR_VERIFIER_ACCOUNT_ID>"
echo
echo "Attestation template (upload to Logos Storage, put its CID above):"
cat "$WORK/attestation.json"

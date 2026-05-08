#!/usr/bin/env bash
# TKT-214.1 — devnet SPL setup for the verifier suite
# ---------------------------------------------------
# One-time helper that:
#   1. (optionally) generates a fresh devnet keypair if $KP doesn't exist
#   2. prints the address so the human can fund it via faucet.solana.com
#   3. waits for SOL to land
#   4. creates a fresh SPL test mint (decimals=6, mirroring USDT-SPL)
#   5. creates the operator's own ATA for that mint
#   6. mints 1,000,000 minor units (= 1.0 token at 6dp) to that ATA
#   7. prints the env exports the verifier test expects:
#        TKT_214_1_DEVNET_FUNDED_PRIVKEY_HEX  — sender wallet's 32-byte seed (hex)
#        TKT_214_2_DEVNET_TEST_MINT           — mint address (base58)
#
# Run:
#     ./architecture/solana/scripts/setup-devnet-spl.sh
#
# Then:
#     eval "$(./architecture/solana/scripts/setup-devnet-spl.sh --env-only)"
#     cd qictrader-backend-rs
#     cargo test --test tkt_214_1_solana_devnet_verify -- \
#         --ignored --nocapture check_1 check_2

set -euo pipefail

DEVNET_RPC="https://api.devnet.solana.com"
KP="${TKT_214_VERIFIER_KEYPAIR:-/tmp/tkt-214-1-verifier.json}"
MIN_FUND_SOL="0.5"
MINT_DECIMALS=6
MINT_AMOUNT_MINOR=1000000  # 1.0 token at 6dp — enough for ~1000 verification runs

# --- argv ---------------------------------------------------------------
ENV_ONLY=0
if [[ "${1:-}" == "--env-only" ]]; then
    ENV_ONLY=1
fi

err() { printf 'setup-devnet-spl: %s\n' "$*" >&2; }

# --- preflight ---------------------------------------------------------
for tool in solana solana-keygen spl-token jq python3; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        err "$tool not on PATH; install Solana toolchain (solana.com/docs/cli/install)"
        exit 1
    fi
done

# --- ensure keypair ----------------------------------------------------
if [[ ! -f "$KP" ]]; then
    [[ $ENV_ONLY -eq 1 ]] || err "creating fresh devnet keypair at $KP"
    solana-keygen new --no-bip39-passphrase --silent --force --outfile "$KP" >/dev/null
fi
ADDR=$(solana-keygen pubkey "$KP")

# Convert solana-keygen JSON (64-byte secret-key array) into the 32-byte
# secret seed that derive_sol/SigningKey::from_bytes expects, then hex.
PRIV_HEX=$(python3 -c "
import json, sys
with open('$KP') as f:
    arr = json.load(f)
# solana-keygen stores 64 bytes: secret_seed (32) + public_key (32)
secret = bytes(arr[:32])
print(secret.hex())
")

# --- balance check / faucet prompt -------------------------------------
SOL_BAL=$(solana --url "$DEVNET_RPC" --keypair "$KP" balance | awk '{print $1}')
SOL_BAL_OK=$(python3 -c "print(1 if float('$SOL_BAL') >= float('$MIN_FUND_SOL') else 0)")

if [[ "$SOL_BAL_OK" -ne 1 ]]; then
    if [[ $ENV_ONLY -eq 1 ]]; then
        err "wallet $ADDR holds $SOL_BAL SOL; need >= $MIN_FUND_SOL — fund first"
        exit 1
    fi
    cat >&2 <<EOF

--------------------------------------------------------------------------------
TKT-214.1 verifier wallet needs funding
--------------------------------------------------------------------------------
  Address: $ADDR
  Current: $SOL_BAL SOL  (need at least $MIN_FUND_SOL)

  1. Open  https://faucet.solana.com
  2. Pick  Devnet
  3. Paste the address above
  4. Drop ~1 SOL (enough for many verification runs + ATA rent)

Press <Return> here once the drop confirms in the faucet UI.
--------------------------------------------------------------------------------
EOF
    read -r _
    # Re-check, with a brief poll to absorb finality lag.
    for _ in 1 2 3 4 5 6 7 8 9 10; do
        SOL_BAL=$(solana --url "$DEVNET_RPC" --keypair "$KP" balance | awk '{print $1}')
        SOL_BAL_OK=$(python3 -c "print(1 if float('$SOL_BAL') >= float('$MIN_FUND_SOL') else 0)")
        if [[ "$SOL_BAL_OK" -eq 1 ]]; then
            break
        fi
        sleep 3
    done
    if [[ "$SOL_BAL_OK" -ne 1 ]]; then
        err "wallet $ADDR still holds only $SOL_BAL SOL after waiting; aborting"
        exit 1
    fi
fi
[[ $ENV_ONLY -eq 1 ]] || err "verifier wallet $ADDR funded with $SOL_BAL SOL"

# --- mint creation (idempotent via cache file) -------------------------
MINT_CACHE="${KP%.json}.mint"
if [[ -f "$MINT_CACHE" ]]; then
    MINT=$(cat "$MINT_CACHE")
    [[ $ENV_ONLY -eq 1 ]] || err "reusing cached mint $MINT (delete $MINT_CACHE to recreate)"
else
    [[ $ENV_ONLY -eq 1 ]] || err "creating fresh devnet SPL mint (decimals=$MINT_DECIMALS)"
    MINT_OUT=$(spl-token --url "$DEVNET_RPC" --fee-payer "$KP" --mint-authority "$KP" \
        create-token --decimals "$MINT_DECIMALS" --output json)
    MINT=$(echo "$MINT_OUT" | jq -r '.commandOutput.address')
    if [[ -z "$MINT" || "$MINT" == "null" ]]; then
        err "failed to parse mint address from spl-token output:"
        err "$MINT_OUT"
        exit 1
    fi
    echo "$MINT" > "$MINT_CACHE"
    [[ $ENV_ONLY -eq 1 ]] || err "mint created: $MINT"

    # Create the operator's own ATA for this mint
    spl-token --url "$DEVNET_RPC" --fee-payer "$KP" --owner "$KP" \
        create-account "$MINT" >/dev/null
    [[ $ENV_ONLY -eq 1 ]] || err "operator ATA created for mint $MINT"

    # Mint a stockpile so re-runs don't deplete the balance
    spl-token --url "$DEVNET_RPC" --fee-payer "$KP" --mint-authority "$KP" \
        mint "$MINT" \
        "$(python3 -c "print($MINT_AMOUNT_MINOR / (10 ** $MINT_DECIMALS))")" \
        >/dev/null
    [[ $ENV_ONLY -eq 1 ]] || err "minted $MINT_AMOUNT_MINOR minor units to operator ATA"
fi

# --- emit env --------------------------------------------------------
cat <<EOF
export TKT_214_1_DEVNET_FUNDED_PRIVKEY_HEX="$PRIV_HEX"
export TKT_214_2_DEVNET_TEST_MINT="$MINT"
EOF

if [[ $ENV_ONLY -eq 0 ]]; then
    cat >&2 <<EOF

--------------------------------------------------------------------------------
Done. Now run:

    eval "\$(./architecture/solana/scripts/setup-devnet-spl.sh --env-only)"
    cd qictrader-backend-rs
    cargo test --test tkt_214_1_solana_devnet_verify -- \\
        --ignored --nocapture check_1 check_2
--------------------------------------------------------------------------------
EOF
fi

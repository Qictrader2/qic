#!/usr/bin/env bash
# Query Heroku Postgres for an escrow by deposit/release/refund tx hash.
# Usage: ./scripts/query-escrow-by-tx.sh [TX_HASH]
# Example: ./scripts/query-escrow-by-tx.sh TK91EDEdTBXvKbpvavyoPeTWUBBWKf3bEt
set -e
TX="${1:-TK91EDEdTBXvKbpvavyoPeTWUBBWKf3bEt}"
APP="${HEROKU_APP:-qictrader-backend-rs}"
echo "Querying escrows for tx_hash = $TX (app=$APP)"
heroku pg:psql -a "$APP" -c "
SELECT id, trade_id, offer_id, buyer_id, seller_id, status,
       deposit_tx_hash, release_tx_hash, refund_tx_hash, created_at
FROM escrows
WHERE deposit_tx_hash = '$TX' OR release_tx_hash = '$TX' OR refund_tx_hash = '$TX';
"

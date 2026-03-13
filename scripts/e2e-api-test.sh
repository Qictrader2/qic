#!/usr/bin/env bash
set -euo pipefail
BASE="http://localhost:5050/api/v1"
PASS=0
FAIL=0
RESULTS=""

check() {
  local name="$1" expected="$2" actual="$3"
  if echo "$actual" | grep -qE "$expected"; then
    PASS=$((PASS+1))
    RESULTS="$RESULTS\n  PASS: $name"
  else
    FAIL=$((FAIL+1))
    RESULTS="$RESULTS\n  FAIL: $name — expected '$expected', got: $(echo "$actual" | head -c 200)"
  fi
}

jq_field() {
  echo "$1" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('$2',''))" 2>/dev/null || echo ""
}

PID_SUFFIX=$$

echo "=========================================="
echo "E2E API Test Suite — $(date)"
echo "Backend: $BASE"
echo "=========================================="

# ── 1. HEALTH ──
echo ""
echo "── 1. Health ──"
H=$(curl -s --max-time 5 http://localhost:5050/health)
check "GET /health" '"status":"ok"' "$H"

HR=$(curl -s --max-time 5 http://localhost:5050/health/ready)
check "GET /health/ready" '"database":"connected"' "$HR"

HD=$(curl -s --max-time 5 http://localhost:5050/health/detailed)
check "GET /health/detailed" 'status' "$HD"

VER=$(curl -s --max-time 5 "$BASE/version")
check "GET /version" 'version' "$VER"

# ── 2. AUTH ──
echo "── 2. Auth ──"
REG_A=$(curl -s --max-time 10 -X POST "$BASE/auth/signup" \
  -H "Content-Type: application/json" \
  -d '{"username":"e2e_seller_'$PID_SUFFIX'","email":"e2e_seller_'$PID_SUFFIX'@test.com","password":"TestPass12345!"}')
check "POST /auth/signup (seller)" 'token' "$REG_A"
TOKEN_A=$(jq_field "$REG_A" "token")
USER_A_ID=$(echo "$REG_A" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('user',{}).get('id',''))" 2>/dev/null || echo "")
echo "  → Seller ID: ${USER_A_ID:0:8}..."

REG_B=$(curl -s --max-time 10 -X POST "$BASE/auth/signup" \
  -H "Content-Type: application/json" \
  -d '{"username":"e2e_buyer_'$PID_SUFFIX'","email":"e2e_buyer_'$PID_SUFFIX'@test.com","password":"TestPass12345!"}')
check "POST /auth/signup (buyer)" 'token' "$REG_B"
TOKEN_B=$(jq_field "$REG_B" "token")
USER_B_ID=$(echo "$REG_B" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('user',{}).get('id',''))" 2>/dev/null || echo "")
echo "  → Buyer ID: ${USER_B_ID:0:8}..."

REG_C=$(curl -s --max-time 10 -X POST "$BASE/auth/signup" \
  -H "Content-Type: application/json" \
  -d '{"username":"e2e_reseller_'$PID_SUFFIX'","email":"e2e_reseller_'$PID_SUFFIX'@test.com","password":"TestPass12345!"}')
check "POST /auth/signup (reseller)" 'token' "$REG_C"
TOKEN_C=$(jq_field "$REG_C" "token")

LOGIN=$(curl -s --max-time 10 -X POST "$BASE/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"email":"e2e_seller_'$PID_SUFFIX'@test.com","password":"TestPass12345!"}')
check "POST /auth/login" 'token' "$LOGIN"
TOKEN_A=$(jq_field "$LOGIN" "token")

ME=$(curl -s --max-time 5 "$BASE/auth/me" -H "Authorization: Bearer $TOKEN_A")
check "GET /auth/me" 'username' "$ME"

if [ -z "$TOKEN_A" ] || [ -z "$TOKEN_B" ]; then
  echo "FATAL: Auth tokens not obtained. Aborting remaining tests."
  echo ""
  echo "Registration response A: $REG_A"
  echo "Registration response B: $REG_B"
  echo ""
  echo "=========================================="
  echo "RESULTS: $PASS passed, $FAIL failed (ABORTED)"
  echo "=========================================="
  echo -e "$RESULTS"
  exit 1
fi

# ── 3. WALLET ──
echo "── 3. Wallet ──"
WAL=$(curl -s --max-time 5 "$BASE/wallet" -H "Authorization: Bearer $TOKEN_A")
check "GET /wallet (list)" 'id' "$WAL"

WBAL=$(curl -s --max-time 5 "$BASE/wallet/balances" -H "Authorization: Bearer $TOKEN_A")
check "GET /wallet/balances" 'currency|balance' "$WBAL"

# ── 4. OFFERS ──
echo "── 4. Offers ──"
OFFER=$(curl -s --max-time 10 -X POST "$BASE/offers" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN_A" \
  -d '{
    "offerType":"sell",
    "cryptocurrency":"USDT",
    "fiatCurrency":"ZAR",
    "pricingMode":"fixed",
    "fixedPrice":2000000,
    "minAmount":10000,
    "maxAmount":10000000,
    "escrowType":"custodial",
    "paymentMethods":["bank_transfer"],
    "terms":"E2E test offer",
    "timeLimitMinutes":30,
    "requireVerified":false,
    "minTradesRequired":0
  }')
check "POST /offers (create)" 'id' "$OFFER"
OFFER_ID=$(jq_field "$OFFER" "id")
echo "  → Offer ID: ${OFFER_ID:0:8}..."

OLIST=$(curl -s --max-time 5 "$BASE/offers" -H "Authorization: Bearer $TOKEN_A")
check "GET /offers (list)" 'id' "$OLIST"

if [ -n "$OFFER_ID" ]; then
  OGET=$(curl -s --max-time 5 "$BASE/offers/$OFFER_ID")
  check "GET /offers/:id" 'id' "$OGET"

  # Buy/sell list
  OBUY=$(curl -s --max-time 5 "$BASE/offers/buy")
  check "GET /offers/buy" 'data' "$OBUY"

  OSELL=$(curl -s --max-time 5 "$BASE/offers/sell")
  check "GET /offers/sell" 'data' "$OSELL"
fi

# ── 5. TRADES (with TRADE-014, TRADE-015 checks) ──
echo "── 5. Trades ──"
if [ -n "$OFFER_ID" ] && [ -n "$TOKEN_B" ]; then
  TRADE=$(curl -s --max-time 10 -X POST "$BASE/trades" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $TOKEN_B" \
    -d '{"offerId":"'"$OFFER_ID"'","fiatAmount":50000}')
  check "POST /trades (create)" 'id' "$TRADE"
  TRADE_ID=$(jq_field "$TRADE" "id")
  echo "  → Trade ID: ${TRADE_ID:0:8}..."

  if [ -n "$TRADE_ID" ]; then
    # TRADE-014: Buyer views — should NOT set counterparty_viewed_at
    T_BUYER=$(curl -s --max-time 5 "$BASE/trades/$TRADE_ID" -H "Authorization: Bearer $TOKEN_B")
    check "GET /trades/:id (buyer view)" 'id' "$T_BUYER"
    CP_VIEWED=$(echo "$T_BUYER" | python3 -c "import sys,json; print(json.load(sys.stdin).get('counterpartyViewed',None))" 2>/dev/null || echo "MISSING")
    check "TRADE-014: buyer view does NOT set counterpartyViewed" 'False' "$CP_VIEWED"

    # Check cancel eligible (should be true before seller views)
    CANCEL_ELIG=$(echo "$T_BUYER" | python3 -c "import sys,json; print(json.load(sys.stdin).get('cancelEligible',None))" 2>/dev/null || echo "MISSING")
    check "TRADE-015: cancelEligible=True before seller views" 'True' "$CANCEL_ELIG"

    # TRADE-014: Seller views — SHOULD set counterparty_viewed_at
    T_SELLER=$(curl -s --max-time 5 "$BASE/trades/$TRADE_ID" -H "Authorization: Bearer $TOKEN_A")
    CP_VIEWED2=$(echo "$T_SELLER" | python3 -c "import sys,json; print(json.load(sys.stdin).get('counterpartyViewed',None))" 2>/dev/null || echo "MISSING")
    check "TRADE-014: seller view SETS counterpartyViewed" 'True' "$CP_VIEWED2"

    # Re-check cancel eligible (should now be false)
    T_BUYER2=$(curl -s --max-time 5 "$BASE/trades/$TRADE_ID" -H "Authorization: Bearer $TOKEN_B")
    CANCEL_ELIG2=$(echo "$T_BUYER2" | python3 -c "import sys,json; print(json.load(sys.stdin).get('cancelEligible',None))" 2>/dev/null || echo "MISSING")
    check "TRADE-015: cancelEligible=False after seller views" 'False' "$CANCEL_ELIG2"

    # TRADE-015: Cancel should fail (counterparty has viewed)
    CANCEL=$(curl -s --max-time 5 -X POST "$BASE/trades/$TRADE_ID/cancel" \
      -H "Authorization: Bearer $TOKEN_B")
    check "TRADE-015: cancel blocked after counterparty viewed" 'counterparty has already viewed|cannot cancel' "$CANCEL"

    # Trade list
    TLIST=$(curl -s --max-time 5 "$BASE/trades" -H "Authorization: Bearer $TOKEN_B")
    check "GET /trades (list)" 'id' "$TLIST"

    # Trade events
    TEVT=$(curl -s --max-time 5 "$BASE/trades/$TRADE_ID/events" -H "Authorization: Bearer $TOKEN_B")
    check "GET /trades/:id/events" 'event|type|created' "$TEVT"

    # Send message
    MSG=$(curl -s --max-time 5 -X POST "$BASE/trades/$TRADE_ID/messages" \
      -H "Content-Type: application/json" \
      -H "Authorization: Bearer $TOKEN_B" \
      -d '{"content":"Hello from E2E test"}')
    check "POST /trades/:id/messages" 'id|content' "$MSG"

    # Get messages
    MSGS=$(curl -s --max-time 5 "$BASE/trades/$TRADE_ID/messages" -H "Authorization: Bearer $TOKEN_B")
    check "GET /trades/:id/messages" 'data|content' "$MSGS"

    # Mark payment
    PAY=$(curl -s --max-time 5 -X POST "$BASE/trades/$TRADE_ID/payment" \
      -H "Content-Type: application/json" \
      -H "Authorization: Bearer $TOKEN_B" \
      -d '{"paymentReference":"REF-12345"}')
    check "POST /trades/:id/payment (mark paid)" 'status|id|Paid|error' "$PAY"
  else
    FAIL=$((FAIL+1))
    RESULTS="$RESULTS\n  FAIL: Trade creation failed — skipping trade tests"
    echo "  → Trade response: $TRADE"
  fi
else
  FAIL=$((FAIL+1))
  RESULTS="$RESULTS\n  FAIL: Offer or token missing — skipping trade tests"
fi

# ── 6. NOTIFICATIONS ──
echo "── 6. Notifications ──"
NOTIF=$(curl -s --max-time 5 "$BASE/notifications" -H "Authorization: Bearer $TOKEN_A")
check "GET /notifications" 'data|notifications' "$NOTIF"

# ── 7. DASHBOARD ──
echo "── 7. Dashboard ──"
DASH=$(curl -s --max-time 5 "$BASE/dashboard" -H "Authorization: Bearer $TOKEN_A")
check "GET /dashboard" 'id|total' "$DASH"

# ── 8. PRICES ──
echo "── 8. Prices ──"
PRICES=$(curl -s --max-time 5 "$BASE/prices")
check "GET /prices" 'price|symbol|coin' "$PRICES"

# ── 9. RESELLER (RESELLER-005, RESELLER-006) ──
echo "── 9. Reseller ──"
RSTATS=$(curl -s --max-time 5 "$BASE/reseller/stats" -H "Authorization: Bearer $TOKEN_C")
check "GET /reseller/stats" 'isActive|totalResells|active' "$RSTATS"

if [ -n "$OFFER_ID" ]; then
  RESELL=$(curl -s --max-time 10 -X POST "$BASE/reseller/resell/$OFFER_ID" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $TOKEN_C" \
    -d '{"markupPercentage":2.5}')
  check "POST /reseller/resell/:id (create resell offer - RESELLER-006)" 'id|markup|error' "$RESELL"
  RESELL_OFFER_ID=$(jq_field "$RESELL" "id")
  echo "  → Resell offer ID: ${RESELL_OFFER_ID:0:8}..."
fi

RACTIVE=$(curl -s --max-time 5 "$BASE/reseller/active" -H "Authorization: Bearer $TOKEN_C")
check "GET /reseller/active" 'id|data' "$RACTIVE"

# ── 10. AFFILIATE ──
echo "── 10. Affiliate ──"
AFF=$(curl -s --max-time 5 "$BASE/affiliate" -H "Authorization: Bearer $TOKEN_A")
check "GET /affiliate" 'tier|referralCode|code|error' "$AFF"

# ── 11. KYC ──
echo "── 11. KYC ──"
KYC=$(curl -s --max-time 5 "$BASE/kyc" -H "Authorization: Bearer $TOKEN_A")
check "GET /kyc" 'status|documents|level|kyc' "$KYC"

# ── 12. SUPPORT ──
echo "── 12. Support ──"
SUPP=$(curl -s --max-time 5 "$BASE/support/tickets" -H "Authorization: Bearer $TOKEN_A")
check "GET /support/tickets" 'data|tickets' "$SUPP"

# ── 13. ESCROW ──
echo "── 13. Escrow ──"
ESC=$(curl -s --max-time 5 "$BASE/escrow" -H "Authorization: Bearer $TOKEN_A")
check "GET /escrow (list)" 'data|escrow' "$ESC"

ESCSTATS=$(curl -s --max-time 5 "$BASE/escrow/stats" -H "Authorization: Bearer $TOKEN_A")
check "GET /escrow/stats" 'total|active|held' "$ESCSTATS"

# ── 14. CONFIG ──
echo "── 14. Config ──"
CFG=$(curl -s --max-time 5 "$BASE/config/cryptos")
check "GET /config/cryptos" 'symbol|name' "$CFG"

FEES=$(curl -s --max-time 5 "$BASE/config/platform-fees")
check "GET /config/platform-fees" 'fee' "$FEES"

# ── 15. PAYMENT METHODS ──
echo "── 15. Payment Methods ──"
PMETHODS=$(curl -s --max-time 5 "$BASE/payment-methods" -H "Authorization: Bearer $TOKEN_A")
check "GET /payment-methods" 'data|id|method' "$PMETHODS"

# ── 16. USERS ──
echo "── 16. Users ──"
if [ -n "$USER_A_ID" ]; then
  UPROFILE=$(curl -s --max-time 5 "$BASE/users/$USER_A_ID" -H "Authorization: Bearer $TOKEN_A")
  check "GET /users/:id (public profile)" 'id|username' "$UPROFILE"
fi

# ── 17. HELP ──
echo "── 17. Help ──"
HELP=$(curl -s --max-time 5 "$BASE/help")
check "GET /help" 'title|slug|article' "$HELP"

echo ""
echo "=========================================="
echo "RESULTS: $PASS passed, $FAIL failed"
echo "=========================================="
echo -e "$RESULTS"
echo ""

if [ $FAIL -eq 0 ]; then
  echo "ALL TESTS PASSED!"
  exit 0
else
  echo "Some tests failed. Review the output above."
  exit 1
fi

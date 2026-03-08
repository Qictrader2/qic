# Detailed steps: Deposit and trade (escrow flow)

This guide covers **trades that use escrow**. The platform supports two flows (MPR-013):

- **Custodial (lock at creation):** When the offer uses **custodial** escrow and the seller has enough balance in their **platform wallet**, the buyer clicks “Buy” and the system **locks the seller’s funds in escrow immediately**. No deposit step — the trade is active and payment details are visible to the buyer right away.
- **On-chain (deposit to address):** When the offer uses **on-chain** escrow (or the seller does not have sufficient platform balance for custodial), the trade is created and the **seller** must **send crypto to an escrow deposit address**. Once the deposit is detected and confirmed, the trade can proceed.

---

## Which flow am I in?

- **You see “Funds secured” / “Escrow funded” as soon as the trade is created** → Custodial lock-at-creation. Skip to [§ 6. After escrow is confirmed](#6-after-escrow-is-confirmed-rest-of-the-trade).
- **You see “Awaiting deposit” / “Send crypto to this address”** → On-chain deposit flow. Follow [§ 2–5](#2-seller-open-the-trade-page) below.

---

## 1. Before the trade: Offer and trade creation

1. **Seller** creates an offer (marketplace → Create offer) with escrow enabled, and sets amount, currency (e.g. USDT), and payment method. For **custodial** offers, the seller should have enough balance in their platform wallet to fund the trade when a buyer clicks “Buy”; otherwise the offer is not fundable and will be rejected at trade creation.
2. **Buyer** finds the offer and clicks **“Buy”** (and confirms amount and payment method).
3. The platform **validates**: offer is active, amount within min/max, and (for custodial) seller has sufficient balance to lock.
4. **Custodial:** The system **locks the seller’s funds in escrow atomically** and creates the trade. The trade is **active** and payment details are visible to the buyer. No deposit step.
5. **On-chain:** A **trade** is created and an **escrow** record is created. The seller will need to **send crypto to the escrow deposit address** (see below).

---

## 2. Seller: Open the trade page (on-chain deposit flow only)

1. **Seller** logs in and goes to **Trade history** (or uses the link from the trade notification).
2. **Seller** opens the **specific trade** (e.g. `/trade/[trade-id]`).
3. On the trade page, the **Escrow** section is shown for the seller with:
   - Status (e.g. “Escrow Not Funded” / “Awaiting deposit”)
   - **Deposit address** (long string and/or QR code)
   - **Exact amount** to send (e.g. `X.XXXXXX USDT`)
   - **Network** (e.g. TRON TRC-20, Solana, Ethereum) — must match exactly.

---

## 3. Seller: Send crypto to the escrow address (on-chain only)

**Option A – Send from an external wallet (another site/app):**

1. In the trade page, **copy the escrow deposit address** (use the copy button next to the address).
2. **Copy the exact amount** (use the copy button next to the amount).
3. Open your **external wallet** (e.g. exchange, MetaMask, TronLink, Phantom) on the **same network** shown (e.g. TRON for USDT TRC-20).
4. Send **exactly that amount** of the **same asset** (e.g. USDT) to the **copied address** on that network.
5. Complete the transfer in the wallet and **save the transaction hash / signature** (you may need it for “Record your transaction hash” if the platform doesn’t detect it).
6. Return to the **QicTrader trade page** and keep it open (or come back to it).

**Option B – Fund from QicTrader wallet (if shown):**

1. If the trade page shows **“Fund from wallet”** or similar and you have enough balance in your QicTrader wallet:
2. Click that button and approve the transfer in the app.
3. The app will send from your in-app wallet to the escrow address and then poll for the balance.

---

## 4. Platform: Balance detection (on-chain only, automatic)

1. While the **trade page is open**, the platform **polls the escrow balance** (about every 10 seconds) by calling the balance API.
2. The **Escrow balance** section on the page will update:
   - “Waiting for deposit...” → no funds yet.
   - “X.XX USDT / Y.YY USDT” and a progress bar → partial or full deposit detected.
   - “Fully funded! Ready to confirm.” → required amount is there.
3. If you sent from an **external wallet**, wait until you see the balance update or “Fully funded” (can take from a few seconds to a couple of minutes depending on the network).

---

## 5. Seller: Confirm the deposit on the platform (on-chain only)

**Normal case (deposit already detected):**

1. When the page shows **“Fully funded! Ready to confirm”** (or similar):
2. Click **“Confirm & Activate Trade”** (or “Check & Confirm Deposit” if it appears first).
3. The platform will mark the escrow as **held** and the trade can proceed. You should see a success message (e.g. “Escrow deposit confirmed! Funds are now secured.”).

**If the balance didn’t update but you already sent:**

1. Expand **“Already sent? Record your transaction hash”**.
2. **Paste the transaction hash / signature** from your wallet (the one you got when you sent the crypto).
3. Click **“Record transaction”**.
4. The platform will send this hash to the backend so the deposit is recorded and the escrow can move to **held**. Then click **“Check & Confirm Deposit”** (or “Confirm & Activate Trade”) if it’s still shown.

---

## 6. After escrow is confirmed: Rest of the trade

1. **Escrow status** on the trade page will show **“Funds Secured in Escrow”** (whether funds were locked at creation or confirmed after deposit).
2. **Buyer** can then **mark the trade as paid** (e.g. after doing the fiat payment) so the platform knows to release.
3. **Seller** (or the system) **releases escrow** so the crypto goes to the buyer.
4. When release completes, the trade can be completed and the flow is done.

---

## 7. Quick reference: Who does what

| Step | Who | Action |
|------|-----|--------|
| Create offer / Start trade | Seller / Buyer | Create offer; buyer starts trade |
| **Custodial:** Lock at creation | Platform | Validates offer + fundable; locks seller balance; trade created with escrow held |
| **On-chain:** Open trade page | Seller | Go to Trade history → open this trade |
| **On-chain:** Send crypto to escrow | **Seller** | Send exact amount to the shown address on the correct network |
| **On-chain:** Wait for balance | Platform | Polls balance every ~10s while trade page is open |
| **On-chain:** Confirm deposit | **Seller** | Click “Confirm & Activate Trade” or “Check & Confirm Deposit” |
| **On-chain:** If deposit not detected | **Seller** | Use “Record your transaction hash” and paste tx hash, then confirm |
| Mark paid / Release | Buyer / Seller | Buyer marks paid; seller (or system) releases escrow |

---

## 8. Troubleshooting

- **“Offer is not fundable right now” when I click Buy:**  
  For **custodial** offers, the seller must have enough balance in their platform wallet to lock for the trade. The seller should add funds or reduce the trade amount. For **on-chain** offers, this message may appear if the offer was configured as custodial but the seller’s balance is insufficient; try an on-chain offer or ask the seller to fund their wallet.

- **Balance stays at zero after I sent (on-chain):**  
  Keep the trade page open a bit longer (polling every 10s). If it still doesn’t update, use **“Already sent? Record your transaction hash”**, paste the tx hash from your wallet, click “Record transaction”, then “Check & Confirm Deposit”.

- **Wrong network or wrong asset:**  
  The deposit will not be detected. Send only the **exact asset** (e.g. USDT) on the **exact network** shown (e.g. TRON TRC-20).

- **I closed the page before confirming (on-chain):**  
  Open the trade again from Trade history. If the balance now shows as funded, click “Confirm & Activate Trade”. If it doesn’t, use “Record your transaction hash” with the tx hash, then confirm.

- **“Record transaction” or “Confirm” fails:**  
  Check your internet connection and try again. If it keeps failing, contact support with your trade ID and (if possible) the transaction hash.

// QuickNode Webhooks custom filter: Tron mainnet inbound USDT to our custodial
// addresses, emitted in the exact shape the backend parses
// (`QuickNodeTronTransfer` in qictrader-backend-rs/src/services/quicknode_webhook.rs):
//
//   [{ "transaction_id", "token_address", "to", "from", "value" }]
//
// Paste everything below into the webhook's "custom filter" editor, dataset
// `block_with_receipts`. Addresses are base58 (T...), transaction ids are bare
// hex, value is a decimal string in token minor units. Returning null tells
// QuickNode there is nothing to deliver for this block, so we are not billed.

const USDT_CONTRACT = "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t";
const WATCH_LIST = "qic_tron_custodial_addresses";
const TRANSFER_TOPIC = "ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

async function main(stream) {
  const out = [];
  for (const block of blocksOf(stream)) {
    for (const receipt of block.receipts || []) {
      if (!succeeded(receipt)) continue;
      for (const log of receipt.logs || []) {
        const transfer = decodeUsdtTransfer(log, receipt);
        if (!transfer) continue;
        if (await qnLib.qnContainsListItem(WATCH_LIST, transfer.to)) out.push(transfer);
      }
    }
  }
  return out.length ? out : null;
}

// `stream.data` is an array of blocks; tolerate one extra level of batching.
function blocksOf(stream) {
  const data = (stream && stream.data) || [];
  return data.flatMap((item) => (Array.isArray(item) ? item : [item]));
}

function succeeded(receipt) {
  const s = receipt.status;
  return s === "0x1" || s === 1 || s === "1" || s === "SUCCESS";
}

function decodeUsdtTransfer(log, receipt) {
  const topics = log.topics || [];
  if (topics.length !== 3 || strip0x(topics[0]).toLowerCase() !== TRANSFER_TOPIC) return null;
  const token = toBase58(log.address);
  if (token !== USDT_CONTRACT) return null;
  const to = toBase58(topics[2].slice(-40));
  const from = toBase58(topics[1].slice(-40));
  const value = hexToDecimal(log.data);
  const txid = strip0x(log.transactionHash || receipt.transactionHash || "").toLowerCase();
  if (!to || !from || value === null || !txid) return null;
  return { transaction_id: txid, token_address: token, to, from, value };
}

function strip0x(hex) {
  return String(hex || "").replace(/^0x/i, "");
}

function hexToDecimal(hex) {
  const h = strip0x(hex);
  if (!/^[0-9a-fA-F]+$/.test(h)) return null;
  return BigInt("0x" + h).toString(10);
}

// Accepts base58 (T...), 41-prefixed hex, or bare 20-byte hex (the
// EVM-style JSON-RPC form QuickNode uses for Tron).
function toBase58(addr) {
  const a = String(addr || "");
  if (/^T[1-9A-HJ-NP-Za-km-z]{33}$/.test(a)) return a;
  let h = strip0x(a).toLowerCase();
  if (h.length === 40) h = "41" + h;
  if (!/^41[0-9a-f]{40}$/.test(h)) return null;
  const payload = hexToBytes(h);
  const check = sha256(sha256(payload)).slice(0, 4);
  return base58(payload.concat(check));
}

function hexToBytes(h) {
  const bytes = [];
  for (let i = 0; i < h.length; i += 2) bytes.push(parseInt(h.substr(i, 2), 16));
  return bytes;
}

const B58 = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
function base58(bytes) {
  let n = BigInt("0x" + bytes.map((b) => b.toString(16).padStart(2, "0")).join(""));
  let s = "";
  while (n > 0n) {
    s = B58[Number(n % 58n)] + s;
    n /= 58n;
  }
  for (const b of bytes) {
    if (b !== 0) break;
    s = "1" + s;
  }
  return s;
}

// Plain SHA-256 over a byte array; the filter runtime has no crypto module.
const K = [
  0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
  0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
  0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
  0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
  0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
  0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
  0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
  0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];
function sha256(bytes) {
  const msg = bytes.slice();
  const bitLen = bytes.length * 8;
  msg.push(0x80);
  while (msg.length % 64 !== 56) msg.push(0);
  for (let i = 7; i >= 0; i--) msg.push(Math.floor(bitLen / 2 ** (8 * i)) & 0xff);
  const H = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
  const W = new Array(64);
  const rotr = (x, n) => (x >>> n) | (x << (32 - n));
  for (let off = 0; off < msg.length; off += 64) {
    for (let i = 0; i < 16; i++) {
      const j = off + i * 4;
      W[i] = ((msg[j] << 24) | (msg[j + 1] << 16) | (msg[j + 2] << 8) | msg[j + 3]) >>> 0;
    }
    for (let i = 16; i < 64; i++) {
      const s0 = rotr(W[i - 15], 7) ^ rotr(W[i - 15], 18) ^ (W[i - 15] >>> 3);
      const s1 = rotr(W[i - 2], 17) ^ rotr(W[i - 2], 19) ^ (W[i - 2] >>> 10);
      W[i] = (W[i - 16] + s0 + W[i - 7] + s1) >>> 0;
    }
    let [a, b, c, d, e, f, g, h] = H;
    for (let i = 0; i < 64; i++) {
      const S1 = rotr(e, 6) ^ rotr(e, 11) ^ rotr(e, 25);
      const ch = (e & f) ^ (~e & g);
      const t1 = (h + S1 + ch + K[i] + W[i]) >>> 0;
      const S0 = rotr(a, 2) ^ rotr(a, 13) ^ rotr(a, 22);
      const maj = (a & b) ^ (a & c) ^ (b & c);
      const t2 = (S0 + maj) >>> 0;
      h = g; g = f; f = e; e = (d + t1) >>> 0;
      d = c; c = b; b = a; a = (t1 + t2) >>> 0;
    }
    H[0] = (H[0] + a) >>> 0; H[1] = (H[1] + b) >>> 0; H[2] = (H[2] + c) >>> 0; H[3] = (H[3] + d) >>> 0;
    H[4] = (H[4] + e) >>> 0; H[5] = (H[5] + f) >>> 0; H[6] = (H[6] + g) >>> 0; H[7] = (H[7] + h) >>> 0;
  }
  const out = [];
  for (const x of H) out.push((x >>> 24) & 0xff, (x >>> 16) & 0xff, (x >>> 8) & 0xff, x & 0xff);
  return out;
}

if (typeof module !== "undefined") module.exports = { main, toBase58, sha256, decodeUsdtTransfer };

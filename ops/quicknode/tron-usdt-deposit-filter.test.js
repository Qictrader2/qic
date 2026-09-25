// Run: node --test ops/quicknode/
const test = require("node:test");
const assert = require("node:assert/strict");
const { main, toBase58, sha256 } = require("./tron-usdt-deposit-filter.js");

const USDT_HEX = "0xa614f803b6fd780986a42c78ec9c7f77e6ded13c";
const WATCHED = "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t";
const TOPIC = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";
const pad = (hex20) => "0x" + "0".repeat(24) + hex20.replace(/^0x/, "");

global.qnLib = {
  qnContainsListItem: async (list, item) => list === "qic_tron_custodial_addresses" && item === WATCHED,
};

function block({ status = "0x1", address = USDT_HEX, to = USDT_HEX, value = "0xa7d8c0" } = {}) {
  return {
    receipts: [
      {
        status,
        transactionHash: "0xABCDEF01",
        logs: [
          {
            address,
            topics: [TOPIC, pad("0x1111111111111111111111111111111111111111"), pad(to)],
            data: "0x" + value.replace(/^0x/, "").padStart(64, "0"),
            transactionHash: "0xABCDEF01",
          },
        ],
      },
    ],
  };
}

test("sha256 matches the FIPS 180-2 'abc' vector", () => {
  const hex = sha256([0x61, 0x62, 0x63]).map((b) => b.toString(16).padStart(2, "0")).join("");
  assert.equal(hex, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
});

test("converts every hex form of the USDT contract to its base58 address", () => {
  assert.equal(toBase58(USDT_HEX), "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t");
  assert.equal(toBase58("41a614f803b6fd780986a42c78ec9c7f77e6ded13c"), "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t");
  assert.equal(toBase58("TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t"), "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t");
  assert.equal(toBase58("0x0000000000000000000000000000000000000000"), "T9yD14Nj9j7xAB4dbGeiX9h8unkKHxuWwb");
  assert.equal(toBase58("not an address"), null);
});

test("emits a watched USDT transfer in the backend's exact shape", async () => {
  const out = await main({ data: [block()] });
  assert.deepEqual(out, [
    {
      transaction_id: "abcdef01",
      token_address: "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t",
      to: WATCHED,
      from: toBase58("0x1111111111111111111111111111111111111111"),
      value: "11000000",
    },
  ]);
});

test("returns null for a recipient that is not on the watch list", async () => {
  assert.equal(await main({ data: [block({ to: "0x2222222222222222222222222222222222222222" })] }), null);
});

test("ignores other tokens and failed transactions", async () => {
  assert.equal(await main({ data: [block({ address: "0x3333333333333333333333333333333333333333" })] }), null);
  assert.equal(await main({ data: [block({ status: "0x0" })] }), null);
});

test("keeps amounts beyond 2^53 exact", async () => {
  const out = await main({ data: [block({ value: "0x" + "f".repeat(20) })] });
  assert.equal(out[0].value, BigInt("0x" + "f".repeat(20)).toString(10));
});

test("tolerates an extra level of batching and empty batches", async () => {
  assert.equal((await main({ data: [[block()]] })).length, 1);
  assert.equal(await main({ data: [] }), null);
  assert.equal(await main({}), null);
});

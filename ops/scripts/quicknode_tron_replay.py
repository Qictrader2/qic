#!/usr/bin/env python3
"""Replay one signed QuickNode Tron webhook delivery against an environment.

Signs the body exactly as QuickNode does (hex HMAC-SHA256 over
nonce + timestamp + body, keyed with the destination security token) so the
endpoint's authentication, chain check and deduplication can be exercised
without a live QuickNode webhook. Staging needs this because QuickNode
Webhooks have no Tron testnet.

The security token is read from QUICKNODE_WEBHOOK_SECRET in the environment
and is never printed. Pull it without echoing, for example:

    export QUICKNODE_WEBHOOK_SECRET="$(heroku config:get QUICKNODE_WEBHOOK_SECRET -a qictrader-backend-staging)"

Usage:

    ops/scripts/quicknode_tron_replay.py --url https://<backend-host>/webhooks/quicknode \\
        --tx <tron tx id> --to <custodial T-address> --from <sender T-address> \\
        --value <USDT minor units> [--token <TRC-20 contract>] [--bad-signature]

Prints the HTTP status and response body.
"""

import argparse
import hashlib
import hmac
import json
import os
import sys
import time
import urllib.error
import urllib.request
import uuid

NILE_USDT = "TXYZopYRdj2D9XRtbG411XZZ3kM5VkAeBf"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--url", required=True)
    parser.add_argument("--tx", required=True)
    parser.add_argument("--to", required=True)
    parser.add_argument("--from", dest="sender", required=True)
    parser.add_argument("--value", required=True, help="token minor units, as a decimal string")
    parser.add_argument("--token", default=NILE_USDT, help="TRC-20 contract (defaults to Nile USDT)")
    parser.add_argument("--bad-signature", action="store_true", help="sign with a wrong key; expect 401")
    args = parser.parse_args()

    secret = os.environ.get("QUICKNODE_WEBHOOK_SECRET", "").strip()
    if not secret:
        print("QUICKNODE_WEBHOOK_SECRET is not set in this shell", file=sys.stderr)
        return 2

    body = json.dumps(
        [
            {
                "transaction_id": args.tx,
                "token_address": args.token,
                "to": args.to,
                "from": args.sender,
                "value": str(args.value),
            }
        ],
        separators=(",", ":"),
    ).encode()
    nonce = uuid.uuid4().hex
    timestamp = str(int(time.time()))
    key = (secret + "-wrong") if args.bad_signature else secret
    signature = hmac.new(key.encode(), nonce.encode() + timestamp.encode() + body, hashlib.sha256).hexdigest()

    request = urllib.request.Request(
        args.url,
        data=body,
        method="POST",
        headers={
            "content-type": "application/json",
            "x-qn-nonce": nonce,
            "x-qn-timestamp": timestamp,
            "x-qn-signature": signature,
        },
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            print(response.status, response.read().decode(errors="replace"))
    except urllib.error.HTTPError as error:
        print(error.code, error.read().decode(errors="replace"))
    return 0


if __name__ == "__main__":
    sys.exit(main())

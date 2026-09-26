#!/usr/bin/env python3
"""Create or repair the QuickNode Tron deposit webhook for one environment.

One run leaves an environment in the state the backend expects:

- the Key-Value Store list holds every custodial Tron address in the lowercase
  0x hex form the Wallet transfers template matches on,
- the backend app has QUICKNODE_API_KEY, QUICKNODE_WEBHOOK_SECRET and
  QUICKNODE_TRON_ADDRESS_LIST, and has restarted with them,
- one active webhook (template evmWalletFilter) posts to the app's
  /webhooks/quicknode with the same security token and watches that list.

Secrets never reach stdout. The API key is read from a 0600 file. The security
token is taken from the app's existing QUICKNODE_WEBHOOK_SECRET, or generated
into a 0600 file on first use. Webhook objects are printed with the token
removed, and `heroku config:set` output (which echoes values) is discarded.

Usage:

    ops/scripts/quicknode_tron_webhook_setup.py --env staging status
    ops/scripts/quicknode_tron_webhook_setup.py --env staging apply
    ops/scripts/quicknode_tron_webhook_setup.py --env staging delete-webhook --id <uuid>

The API key file defaults to ~/.qictrader-secrets/quicknode_api_key_<env>.
Staging and production use different API keys and different security tokens.
"""

import argparse
import hashlib
import json
import os
import re
import secrets
import subprocess
import sys
import time
import urllib.error
import urllib.request

API = "https://api.quicknode.com"
# Cloudflare in front of api.quicknode.com rejects urllib's default User-Agent (error 1010).
USER_AGENT = "qictrader-ops/quicknode-tron-webhook-setup"
TEMPLATE_ID = "evmWalletFilter"
SECRETS_DIR = os.path.expanduser("~/.qictrader-secrets")
NOTIFICATION_EMAIL = "developers@qictrader.com"

ENVIRONMENTS = {
    "staging": {
        "app": "qictrader-backend-staging",
        "url": "https://staging-api.qictrader.com/webhooks/quicknode",
        "health": "https://staging-api.qictrader.com/health",
        "list": "qic_tron_custodial_addresses_staging",
        "name": "qictrader-staging-tron-nile",
        "network": "tron-nile",
    },
    "production": {
        "app": "qictrader-backend-rs",
        "url": "https://api.qictrader.com/webhooks/quicknode",
        "health": "https://api.qictrader.com/health",
        "list": "qic_tron_custodial_addresses",
        "name": "qictrader-production-tron-mainnet",
        "network": "tron-mainnet",
    },
}

B58 = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
TRON_BASE58 = re.compile(r"^T[1-9A-HJ-NP-Za-km-z]{33}$")


class SetupError(Exception):
    pass


# ── Tron address form ────────────────────────────────────────────────────────


def tron_base58_to_watch_item(address: str) -> str:
    """`T...` base58check to the `0x` + 20-byte lowercase hex the template matches."""
    n = 0
    for ch in address:
        n = n * 58 + B58.index(ch)
    raw = n.to_bytes(25, "big")
    body, checksum = raw[:21], raw[21:]
    if hashlib.sha256(hashlib.sha256(body).digest()).digest()[:4] != checksum:
        raise SetupError(f"bad Tron checksum: {address}")
    if body[0] != 0x41:
        raise SetupError(f"not a Tron address: {address}")
    return "0x" + body[1:].hex()


# ── Secrets ──────────────────────────────────────────────────────────────────


def read_secret_file(path: str) -> str:
    try:
        mode = os.stat(path).st_mode & 0o777
    except FileNotFoundError:
        raise SetupError(f"missing {path}")
    if mode & 0o077:
        raise SetupError(f"{path} is readable by others (mode {oct(mode)}); chmod 600 it")
    with open(path) as f:
        value = f.read().strip()
    if not value or any(c.isspace() for c in value):
        raise SetupError(f"{path} does not hold a single token")
    return value


def write_secret_file(path: str, value: str) -> None:
    os.makedirs(os.path.dirname(path), mode=0o700, exist_ok=True)
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
    with os.fdopen(fd, "w") as f:
        f.write(value + "\n")


def redact(text: str, *values: str) -> str:
    for v in values:
        if v:
            text = text.replace(v, "<redacted>")
    return text


# ── Heroku ───────────────────────────────────────────────────────────────────


def heroku_config_get(app: str, name: str) -> str:
    out = subprocess.run(
        ["heroku", "config:get", name, "-a", app], capture_output=True, text=True, check=False
    )
    if out.returncode != 0:
        raise SetupError(f"heroku config:get {name} failed on {app}")
    return out.stdout.strip()


def heroku_config_set(app: str, values: dict) -> None:
    args = ["heroku", "config:set", "-a", app] + [f"{k}={v}" for k, v in values.items()]
    out = subprocess.run(args, capture_output=True, text=True, check=False)
    if out.returncode != 0:
        raise SetupError(
            "heroku config:set failed: " + redact(out.stderr.strip(), *values.values())
        )


def custodial_tron_addresses(app: str) -> list:
    sql = (
        "COPY (SELECT address FROM custodial_wallets "
        "WHERE network = 'tron_mainnet' ORDER BY address) TO STDOUT"
    )
    out = subprocess.run(
        ["heroku", "pg:psql", "-a", app, "-c", sql], capture_output=True, text=True, check=False
    )
    if out.returncode != 0:
        raise SetupError(f"could not read custodial_wallets on {app}: {out.stderr.strip()[-300:]}")
    addresses = [line.strip() for line in out.stdout.splitlines() if TRON_BASE58.match(line.strip())]
    if not addresses:
        raise SetupError(f"no custodial Tron addresses found on {app}")
    return addresses


# ── QuickNode REST ───────────────────────────────────────────────────────────


class QuickNode:
    def __init__(self, api_key: str):
        self.api_key = api_key

    def call(self, method: str, path: str, body=None, ok_missing=False):
        data = None if body is None else json.dumps(body).encode()
        request = urllib.request.Request(
            API + path,
            data=data,
            method=method,
            headers={
                "accept": "application/json",
                "content-type": "application/json",
                "user-agent": USER_AGENT,
                "x-api-key": self.api_key,
            },
        )
        try:
            with urllib.request.urlopen(request, timeout=30) as response:
                text = response.read().decode()
        except urllib.error.HTTPError as error:
            if ok_missing and error.code == 404:
                return None
            detail = redact(error.read().decode(errors="replace")[:500], self.api_key)
            raise SetupError(f"{method} {path} -> HTTP {error.code}: {detail}")
        return json.loads(text) if text.strip() else {}

    def webhooks(self) -> list:
        found, offset = [], 0
        while True:
            page = self.call("GET", f"/webhooks/rest/v1/webhooks?limit=50&offset={offset}")
            data = page.get("data") or []
            found.extend(data)
            total = (page.get("pageInfo") or {}).get("total", len(found))
            offset += len(data)
            if not data or offset >= total:
                return found

    def list_items(self, key: str):
        body = self.call("GET", f"/kv/rest/v1/lists/{key}", ok_missing=True)
        if body is None:
            return None
        for candidate in (body, body.get("data"), (body.get("data") or {}).get("items") if isinstance(body.get("data"), dict) else None, body.get("items")):
            if isinstance(candidate, list):
                return [str(v) for v in candidate]
        return []


def safe_view(webhook: dict, token: str = "") -> dict:
    dest = dict(webhook.get("destination_attributes") or {})
    has_token = bool(dest.pop("security_token", None))
    view = {
        "id": webhook.get("id"),
        "name": webhook.get("name"),
        "network": webhook.get("network"),
        "status": webhook.get("status"),
        "templateId": webhook.get("templateId"),
        "templateArgs": webhook.get("templateArgs"),
        "url": dest.get("url"),
        "compression": dest.get("compression"),
        "has_security_token": has_token,
        "created_at": webhook.get("created_at"),
    }
    if token:
        actual = (webhook.get("destination_attributes") or {}).get("security_token") or ""
        view["token_matches_app"] = secrets.compare_digest(actual, token)
    return view


# ── Steps ────────────────────────────────────────────────────────────────────


def ensure_list(qn: QuickNode, key: str, wanted: list) -> None:
    current = qn.list_items(key)
    if current is None:
        qn.call("POST", "/kv/rest/v1/lists", {"key": key, "items": wanted})
        print(f"list {key}: created with {len(wanted)} addresses")
        return
    missing = sorted(set(wanted) - set(current))
    if missing:
        qn.call("PATCH", f"/kv/rest/v1/lists/{key}", {"addItems": missing})
    print(f"list {key}: {len(current)} before, added {len(missing)}, now covers all {len(wanted)} custodial addresses")


def wait_for_health(url: str, seconds: int = 180) -> None:
    deadline = time.time() + seconds
    while time.time() < deadline:
        try:
            request = urllib.request.Request(url, headers={"user-agent": USER_AGENT})
            with urllib.request.urlopen(request, timeout=10) as response:
                if response.status == 200:
                    return
        except (urllib.error.URLError, TimeoutError):
            pass
        time.sleep(5)
    raise SetupError(f"{url} did not return 200 within {seconds}s")


def unsigned_probe(url: str) -> int:
    request = urllib.request.Request(
        url,
        data=b"{}",
        method="POST",
        headers={"content-type": "application/json", "user-agent": USER_AGENT},
    )
    try:
        with urllib.request.urlopen(request, timeout=20) as response:
            return response.status
    except urllib.error.HTTPError as error:
        return error.code


def resolve_token(env_name: str, app: str) -> tuple:
    """(token, source). Reuses the app's secret so a rerun never rotates it."""
    existing = heroku_config_get(app, "QUICKNODE_WEBHOOK_SECRET")
    if existing:
        return existing, "app"
    path = os.path.join(SECRETS_DIR, f"quicknode_webhook_secret_{env_name}")
    if os.path.exists(path):
        return read_secret_file(path), "file"
    token = secrets.token_hex(32)
    write_secret_file(path, token)
    return token, "generated"


def cmd_status(env_name: str, env: dict, qn: QuickNode) -> None:
    token = heroku_config_get(env["app"], "QUICKNODE_WEBHOOK_SECRET")
    hooks = qn.webhooks()
    print(f"{len(hooks)} webhook(s) on the account:")
    for hook in hooks:
        print(json.dumps(safe_view(hook, token), indent=None))
    items = qn.list_items(env["list"])
    print(f"list {env['list']}: {'missing' if items is None else f'{len(items)} items'}")
    for name in ("QUICKNODE_API_KEY", "QUICKNODE_WEBHOOK_SECRET", "QUICKNODE_TRON_ADDRESS_LIST"):
        value = heroku_config_get(env["app"], name)
        shown = value if name == "QUICKNODE_TRON_ADDRESS_LIST" else ("set" if value else "unset")
        print(f"{env['app']} {name}: {shown}")
    print(f"unsigned POST {env['url']} -> {unsigned_probe(env['url'])} (401 = secret configured, 503 = off)")


def cmd_apply(env_name: str, env: dict, qn: QuickNode, api_key: str, network: str) -> None:
    hooks = qn.webhooks()
    matching = [
        h for h in hooks
        if h.get("name") == env["name"]
        or (h.get("destination_attributes") or {}).get("url") == env["url"]
    ]
    if len(matching) > 1:
        for hook in matching:
            print(json.dumps(safe_view(hook)))
        raise SetupError("more than one webhook targets this environment; delete the extras first")
    if matching and matching[0].get("network") != network:
        print(json.dumps(safe_view(matching[0])))
        raise SetupError(
            f"existing webhook is on {matching[0].get('network')}, not {network}; "
            "a template update cannot move networks, so delete it and rerun"
        )

    wanted = sorted({tron_base58_to_watch_item(a) for a in custodial_tron_addresses(env["app"])})
    ensure_list(qn, env["list"], wanted)

    token, source = resolve_token(env_name, env["app"])
    print(f"security token: reused from {source}" if source != "generated" else "security token: generated (stored 0600)")

    desired = {
        "QUICKNODE_API_KEY": api_key,
        "QUICKNODE_WEBHOOK_SECRET": token,
        "QUICKNODE_TRON_ADDRESS_LIST": env["list"],
    }
    changed = {k: v for k, v in desired.items() if heroku_config_get(env["app"], k) != v}
    if changed:
        heroku_config_set(env["app"], changed)
        print(f"{env['app']}: set {', '.join(sorted(changed))}; waiting for the restart")
        time.sleep(20)
        wait_for_health(env["health"])
    else:
        print(f"{env['app']}: config already correct, no restart")

    status = unsigned_probe(env["url"])
    if status != 401:
        raise SetupError(f"unsigned POST answered {status}, expected 401; not pointing QuickNode at it")

    body = {
        "name": env["name"],
        "notification_email": NOTIFICATION_EMAIL,
        "destination_attributes": {"url": env["url"], "security_token": token, "compression": "none"},
        "templateArgs": {"walletsListName": env["list"]},
    }
    if matching:
        hook_id = matching[0]["id"]
        hook = qn.call("PATCH", f"/webhooks/rest/v1/webhooks/{hook_id}/template/{TEMPLATE_ID}", body)
        print(f"webhook {hook_id}: updated")
    else:
        hook = qn.call(
            "POST", f"/webhooks/rest/v1/webhooks/template/{TEMPLATE_ID}", dict(body, network=network)
        )
        hook_id = hook["id"]
        print(f"webhook {hook_id}: created on {network}")

    hook = qn.call("GET", f"/webhooks/rest/v1/webhooks/{hook_id}")
    if hook.get("status") != "active":
        qn.call("POST", f"/webhooks/rest/v1/webhooks/{hook_id}/activate", {"startFrom": "latest"})
        hook = qn.call("GET", f"/webhooks/rest/v1/webhooks/{hook_id}")

    view = safe_view(hook, token)
    print(json.dumps(view, indent=2))
    if not view.get("token_matches_app") or view.get("status") != "active":
        raise SetupError("webhook is not active with the app's security token")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--env", required=True, choices=sorted(ENVIRONMENTS))
    parser.add_argument("--api-key-file")
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("status")
    apply = sub.add_parser("apply")
    apply.add_argument("--network", help="override the QuickNode network id (tron-nile or tron-mainnet)")
    delete = sub.add_parser("delete-webhook")
    delete.add_argument("--id", required=True)
    args = parser.parse_args()

    env = ENVIRONMENTS[args.env]
    key_file = args.api_key_file or os.path.join(SECRETS_DIR, f"quicknode_api_key_{args.env}")
    try:
        api_key = read_secret_file(key_file)
        qn = QuickNode(api_key)
        if args.command == "status":
            cmd_status(args.env, env, qn)
        elif args.command == "apply":
            cmd_apply(args.env, env, qn, api_key, args.network or env["network"])
        else:
            qn.call("DELETE", f"/webhooks/rest/v1/webhooks/{args.id}")
            print(f"webhook {args.id}: deleted")
    except SetupError as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())

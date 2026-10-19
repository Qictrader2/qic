# Ticket preparation operations

This directory is the versioned source of the Mac mini preparation runner, worker contracts, renderer and targeted tests. Runtime artifacts remain private under `.scratch/ticket-preparation/runtime/`. The historical `.claude/scripts/qic_prepare.py` entrypoint and worker/contract paths are compatibility links installed by `install.py`.

## Check a change

Run `python3 -m unittest discover -s ops/ticket-preparation/tests -p 'test_*.py' -v`. The root `Ticket preparation tooling` GitHub workflow runs this command on changes here. Tests use temporary fixtures; no account credentials, models, browser or database are required.

## Install or update on the Mac

Run `python3 ops/ticket-preparation/install.py`. Existing local files are retained in a dated `.scratch` backup before compatibility links change. The installer preserves the launchd job's environment and logs, points it to the canonical runner and enables a 15-minute wakeup. The runner performs at most three transient-error attempts per Dubai day, separated by at least 15 minutes. A completed scan containing owner blockers does not dispatch models again that day.

## Claude sign-in for workers

Claude workers need a CLI sign-in the runner can reach without a browser or the macOS keychain (launchd and SSH sessions cannot always open it). Use a long-lived token on every machine that runs preparation:

1. Run `claude setup-token` and complete the browser sign-in. It prints a token valid for about a year.
2. Run `bash ops/ticket-preparation/store-claude-token.sh` and paste the token when asked. Input is hidden; the token is written to `~/.config/qic/claude-token.env`, readable only by you (owner-only ACL on Windows), with its creation date, and checked with one CLI call.

The runner loads that file when `CLAUDE_CODE_OAUTH_TOKEN` is not already set, passes the token only to Claude workers and its usage check, and never prints it. `status` reports `claude-token` with its source and days left, and every command warns on stderr from 30 days before the year is up. Revoke a leaked token from the claude.ai account settings, then repeat both steps.

## Run and verify

- `python3 ops/ticket-preparation/qic_prepare.py status` reads compact local receipts without account access.
- `python3 ops/ticket-preparation/qic_prepare.py scan --apply` explicitly prepares Sprint Ready, preserving earlier plans and completed designs.
- `python3 ops/ticket-preparation/qic_prepare.py scan` verifies current attached plans and design bytes against the current card and watched source, without dispatching models.
- `python3 ops/ticket-preparation/qic_prepare.py preflight <ticket>` verifies an individual card. It cannot overwrite the latest nightly report.

Each run has an immutable identity under `runtime/runs/`, with command, timestamps, results, counts and errors. Separate `latest-nightly.json`, `latest-scan.json` and `latest-preflight.json` receipts identify the corresponding operations. `last-scan.json` remains a compatibility receipt for full scans only. An interrupted run retains its running receipt; inspect its process and artifacts before retrying.

## Design recovery and approval

Publishable manifests contain HTML, PNG/PDF, CSS, JSON and Markdown. Python generators are retained locally under `internal_files`. The runner can repair this mechanical manifest error without recreating inspected exports or weakening the format checks. Other failures name the exact file, persist validation feedback and allow one bounded worker repair. Operator inspection remains required for exhausted nonmechanical failures.

The evidence allowlist in `evidence-sources.json` points workers to current local maps and baseline indexes. Markdown sources are copied into the assigned output directory with hashes. This configuration grants no publication permission for unrelated files or credentials.

The same evidence hashes are bound to plans, blocked receipts and preparation identities. A changed map or a newly available baseline index invalidates the old result even when nobody edits the Trello card. Retained pre-upgrade evidence indexes can prove unchanged documents during migration, preserving completed planning and designs.

A missing proposed design is preparation work. A human-approved implementation gate remains attached to a `proposal-ready` plan. `preflight` and `start` report `awaiting-approval`, and `start` cannot move that card. Financial and security questions remain blockers until the owner resolves them; preparation never substitutes a preferred rule for an unapproved policy.

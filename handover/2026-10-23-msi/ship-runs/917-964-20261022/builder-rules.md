# Builder rules for 917 + 964 BOIL (every builder reads this first)

Run dir: /Users/starbird/Workspace/Code/qic/.scratch/jev-qic-ship/917-964-20261022
Backend worktree: /Users/starbird/Workspace/Code/qic/.scratch/964/wt/qictrader-backend-rs (branch 964-password-policy, base origin/main 8fda17fa)
Frontend worktree: /Users/starbird/Workspace/Code/qic/.scratch/964/wt/frontend (branch 964-password-policy, base origin/main a73d2708, node_modules installed)
Plans: <run dir>/PLAN-964.md and <run dir>/PLAN-917.md · Contract: <run dir>/api-contract.md · Project rules: /Users/starbird/Workspace/Code/qic/CLAUDE.md (NO SUPPRESSION, Result types, enums over strings, exhaustive matches, types first).
Where the plans disagree, 964 wins: the server enforces all five rules (owner decision 16 Oct, Christian 21 Oct); 917's "no composition rules on the server" is superseded. The length message is exactly "password must be at least 12 characters".

1. Narrow scope: edit only the files your brief lists. Other builders work in the same worktrees at the same time on other files; never touch theirs. If you need a change in a file you do not own, write the request to <run dir>/requests-<your name>.md and say so in your reply.
2. Keep the tree compiling. A broken build blocks the other builders.
3. No git state changes: never git stash, stash pop, reset, checkout, restore, clean, commit or push. To see the old version of a file use `git show origin/main:<path>`.
4. Cargo: do NOT set CARGO_TARGET_DIR (the shared build dir comes from /Users/starbird/Workspace/Code/qic/.cargo/config.toml). Always `CARGO_INCREMENTAL=0`. "Blocking waiting for file lock" means another build is running; wait. Check `df -h /` before a long build; under 15 GB free, stop and tell L1. Never delete target dirs.
5. Database tests: mint your own DB with `python3 /Users/starbird/Workspace/Code/qic/.claude/skills/jev-qic-ship/scripts/jev_phase_gate.py --mint-test-db --run-dir <run dir> --run-id 917-964-20261022` and run with `DATABASE_URL=postgres://postgres@127.0.0.1:5433/<minted> REQUIRE_TEST_DB=1 RUST_MIN_STACK=2147483648` and `ulimit -s unlimited`. Never run the full `cargo test`; run your own target(s) and `cargo test --lib <module>::`.
6. Write no comments of any kind (no //, ///, //!, /* */, {/* */} in TSX). Meaning goes into names, types, enums and test names.
7. Every test file that names a route or struct you changed must be run (`grep -rln "<name>" tests src`).
8. No `let _ =` on a Result, no `.ok()` to swallow errors, no `_ =>` on domain enums, no unwrap/expect outside tests. Never log or echo a password.
9. New DB tests: run your test target 3 times in a row tee'd to one log before reporting.
10. Heartbeat: append `$(date -u +%FT%TZ) <your name>: <done>, next <next>` to <run dir>/run.log at least every 10 minutes.
11. Slow points: 5 minutes waiting on one command, or 3 tries / 15 minutes on one sub-goal without success, stop and report it (command, seconds, process alive, CPU busy, files changing, what it waits on, attempts) instead of looping.
12. Silly blocker rule (owner, 22 Oct): if an environment problem blocks you, say what environment fix would prevent it.
13. Tee every cargo/bun test run you rely on to <run dir>/<your name>-<what>.log, with the DATABASE_URL echoed first (backend) and `; echo "EXIT=$?"` at the end. Grep each log for FAILED before you claim green.
14. Frontend: use bun only. Never run `bun run dev` (it kills whatever listens on port 3000). For a local preview use `bunx next dev -p <free port>`.

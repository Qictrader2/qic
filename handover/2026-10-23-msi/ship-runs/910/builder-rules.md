# Builder rules for 910 BOIL (every builder reads this first)

Run dir: /Users/starbird/Workspace/Code/qic/.scratch/jev-qic-ship/910
Backend worktree: /Users/starbird/Workspace/Code/qic/.scratch/910/wt/qictrader-backend-rs (branch 910-full-funding-check, base origin/main 8fda17fa, which includes 920)
Frontend: no change in this ticket.
Plan: <run dir>/PLAN.md · Internal contract: <run dir>/api-contract.md · Project rules: /Users/starbird/Workspace/Code/qic/CLAUDE.md (NO SUPPRESSION, Result types, newtypes, enums over strings, exhaustive matches, handlers verify auth.user_id owns the resource).

1. Narrow scope: edit only the files your brief lists. Other builders work in the same backend worktree at the same time on other files; never touch theirs. If you need a change in a file you do not own, write the request to <run dir>/requests-<your name>.md and tell L1 in your reply.
2. Keep the tree compiling. Make each edit compile-safe; a broken build blocks the other builders. Never leave the tree broken for more than a few minutes.
3. No git state changes: never git stash, stash pop, reset, checkout, restore, clean, commit or push. To see the old version of a file use `git show origin/main:<path>`.
4. Cargo: every command uses `CARGO_INCREMENTAL=0` and `CARGO_TARGET_DIR=/Users/starbird/Workspace/Code/qic/.scratch/910/target-910` (this run's private clone; never the shared target, which can link another worktree's code). Below 8 GB free (`df -g /`), pause and tell L1. Never delete target dirs.
5. Database tests: mint your own DB with `python3 /Users/starbird/Workspace/Code/qic/.claude/skills/jev-qic-ship/scripts/jev_phase_gate.py --mint-test-db --run-dir <run dir> --run-id 910-r1` and run with `DATABASE_URL=postgres://postgres@127.0.0.1:5433/<minted> REQUIRE_TEST_DB=1 RUST_MIN_STACK=2147483648` and `ulimit -s unlimited`. Never run the full `cargo test`; run your own test target(s) and `cargo test --lib <module>::`. A migration you edit after it was applied needs a fresh DB.
6. Write no comments of any kind (no //, ///, //!, /* */, -- in SQL, {/* */} in TSX). Meaning goes into names, types, enums and test names.
7. Every test file that names a route or function you changed must be run (`grep -rln "<name>" tests src`).
8. No quiet fallbacks on money paths; no `let _ =` on a Result, no `.ok()` on financial paths, no `_ =>` on domain enums, no unwrap/expect outside tests.
9. Docs move with code: if you change lifecycle, ledger or replay behaviour, update docs/intended-entity-state-machines.md / docs/as-built/money-flow.md in the same pass only if your brief assigns docs to you; otherwise list the doc change needed in your reply.
10. New async/DB tests: run your test target 3 times in a row tee'd to one log before reporting (10 if a test is concurrency-based and fast).
11. Heartbeat: append `$(date -u +%FT%TZ) <your name>: <done>, next <next>` to <run dir>/run.log at least every 10 minutes.
12. Slow points: 5 minutes waiting on one command, or 3 tries / 15 minutes on one sub-goal without success, stop and report it (command, seconds, process alive, CPU busy, files changing, what it waits on, attempts) in your reply instead of looping.
13. Silly blocker rule (owner, 22 Oct): if an environment problem blocks you (missing tool, stale path, lock), say what environment fix would prevent it, in your reply.
14. Tee every cargo/bun test run you rely on to a log in <run dir> named <your name>-<what>.log, with the DATABASE_URL echoed first and `; echo "EXIT=$?"` at the end. Report test result lines from those logs, and grep each log for FAILED before you claim green.
15. Suppression scans cover untracked new files too: `{ git diff origin/main -- src; git ls-files --others --exclude-standard -- src | xargs cat; } | grep -nE "let _ =|\.ok\(\);|_ =>|unwrap\(\)|expect\("`.
16. Owner rule (Christian, PM, 21 Oct): check and lock the full amount including the fee in one transaction on every trade-start route; a failed attempt leaves no trade, no hold and no payment instruction (no system "funds held in escrow" message, no payment-details message).

# How to build one 906 page-group slice

You are the overseer for ONE slice (for example 959-S1) of the QicTrader customer redesign. You build it, gate it, get it
reviewed and open its PR into `redesign/906`. You do NOT merge: the coordinator merges in schedule order.

## Read first
1. Your card's plan: `/Users/starbird/Workspace/Code/qic/.scratch/<card>/reports/plan.md`. Your slice's section is your spec.
2. `/Users/starbird/Workspace/Code/qic/.scratch/906-redesign/BUILD-SCHEDULE.md` section 1 (file ownership) and the
   decisions in section 3. Touch ONLY the files your slice owns or appends to there. If you need a file another slice
   owns, stop and report instead of editing it.
3. The v5.2 boards your slice names, in `~/Downloads/906-design-pack-v5.2/` (PDFs; the v5.2 sources and `CHANGES.md`
   live there too; tokens in `v5.2-src/shared/qt-v5.css`). The boards are the spec: replicate them exactly
   (sizes, spacing, copy, states, both 390 and the wide frames). Card text loses to the boards.
4. `/Users/starbird/Workspace/Code/qic/DESIGN.md` and the shared 906 pieces already on `redesign/906` (customer surface,
   `customer:` variant, shared controls, Amount, StatusChip, ErrorState, dialog shell, toasts at 6s, dock/island,
   `--qt-overlay-bottom`). Reuse them; never rebuild them.
5. `/Users/starbird/Workspace/Code/qic/.scratch/906-redesign/HANDOVER.md` owner decisions log.

## Owner rules
- Button icons (since #327): customer sm and default 18px, lg and icon 20px. Where your board draws another size, add
  `customer:[&_svg]:size-*` on that Button and test it.
- Decisions (owner, 22 Oct): never ask Marcello. Card vs board: the board wins. Board vs board: the board drawn for that
  exact screen or state wins over shared or reference boards, else the majority of frames. Backend: read backend
  origin/main (/Users/starbird/Workspace/Code/qic/qictrader-backend-rs, `git show refs/remotes/origin/main:<path>`); build
  against what it returns today. Log every such call in your report with the boards used.
- Backend-dependent behaviour: build against today's API. Leave the dependent part out (or show the boards' honest
  fallback) and list the backend card in your report. Never change backend code.
- No colours, fonts, radii or shadows outside DESIGN.md / qt tokens; no hardcoded hex in components.
- Tests: write focused unit tests for every behaviour you add (vitest/bun as the repo does). Specific assertions only.
- Frontend rules from /Users/starbird/Workspace/Code/qic/CLAUDE.md apply (no empty catches, no silent test passes).

## Worktree
- Create it yourself:
  `git -C /Users/starbird/Workspace/Code/qic/frontend fetch -q origin` then
  `git -C /Users/starbird/Workspace/Code/qic/frontend worktree add /Users/starbird/Workspace/Code/qic/.scratch/906-redesign/<slice>-root/frontend -b <card>-<slice-lowercase>-<short-name> refs/remotes/origin/redesign/906`
  (always full refs; zsh does not word-split). Then `bun install` inside it. Never symlink node_modules.
- Never stash, hard-reset, discard, clean, switch branches in someone else's worktree, push to main, or force-push.

## Gate (one at a time on this machine)
- Take the lock: `until mkdir /Users/starbird/Workspace/Code/qic/.scratch/906-redesign/.gate-lock 2>/dev/null; do sleep 20; done`
  (if the lock dir is older than 40 minutes, it is stale: remove it). Release with `rmdir` right after the gate.
- Run: `QIC_GATE_ROOT=/Users/starbird/Workspace/Code/qic/.scratch/906-redesign/<slice>-root /Users/starbird/Workspace/Code/qic/.claude/scripts/gate.sh --base refs/remotes/origin/redesign/906`
  plus your slice's unit tests (`bun run test <paths>` or the repo's runner). The gate must pass.

## Budget clock
`python3 /Users/starbird/Workspace/Code/qic/.claude/scripts/qic_ship.py budget start --ticket <slice> --size M --file /Users/starbird/Workspace/Code/qic/.scratch/<card>/budget-<slice>.json`
(size S for slices under ~400 lines, L for over ~1,200). Mark phases with `budget phase --name BOIL|TEMPER|PUSH`.
Run `budget end` when the PR is open.

## Review
- Dispatch an independent reviewer (subagent_type "opus55-high", read-only) with the diff, the plan section and the
  board PDFs. It returns APPROVE or NEEDS_FIXES with findings. Fix every HIGH and MEDIUM, re-gate, re-review.
- Slices marked "two sign-offs" in BUILD-SCHEDULE.md need a second independent opus55-high review that approves.

## Commit, push, PR
- Commit message: `<card>: <Short description> (<slice>)`. No trailers, no AI attribution, no emoji.
- Before pushing: `git fetch -q origin` and rebase onto `refs/remotes/origin/redesign/906`; re-gate if anything changed.
- `git push -u origin <branch>` (never --force). Open the PR with
  `gh pr create --repo Qictrader2/Frontend --base redesign/906 --head <branch>` . Title `<card>: ... (<slice>)`.
  Body in the first person, no em dashes, no AI attribution: what I built, which boards, what I left for which backend
  card, test and gate evidence, screenshots path if any.
- Do NOT merge. Do NOT touch Trello.

## Dispatching workers (environment fix, 22 Oct)
- Dispatch every builder, fixer and reviewer with `run_in_background: false` and wait for its result in the same call.
  Background workers' completion notices reach the top session, not you, and file watchers fire early or expire, so a
  background worker leaves you stalled. If a worker you already started in the background is still running, wait for it
  once; every later dispatch is foreground.
- Silly blocker rule (owner, 22 Oct): when something trivial blocks you (a watcher firing early, a cold build tripping a
  time limit, a missing tool, a stale path), read /Users/starbird/.claude/skills/retro/SKILL.md, find the environment
  fix (a check, a pointer, tooling or information access), implement it, note it in your report, then retry.

## Machine limits and known failures (22 Oct, after a restart)
- Hold the gate lock only for gate.sh and jest; never for browser checks or e2e runs (another session will clear a
  long-held lock as stale). Run Puppeteer and e2e with `PUPPETEER_CACHE_DIR=$HOME/.cache/puppeteer`.
- The build step sometimes fails fetching Google Fonts (Figtree, Poppins). That is network, not your code: re-run the
  gate once; only a second identical failure counts.
- Never kill processes by name (`pkill -f next-server`, `killall node`): other sessions' servers die too. Record the
  PID of anything you start and kill only that PID.
- Browser probes: close every tab in a finally block, even when the script crashes; a tab left open signs the QA
  session out.
- Disk: before your first install or build run `df -h /`; under 12 GB free, stop and report to the coordinator instead of building (the coordinator tells the parent session before pausing anything). After
  your gate passes, delete your worktree's `.next` (it is rebuilt by the next gate). The coordinator removes your
  worktree after the PR merges; never leave anything you need only inside it (reports go under .scratch/<card>/).
- This Mac has 16 GB. At most 3 heavy jobs (builds, jest runs, gate) run at once across ALL 906 work. Run jest with
  `--maxWorkers=2`. A builder may run tests only while holding the gate lock, or one targeted suite at a time.
- A `.gate-lock` older than 40 minutes is stale (a crash or restart left it): first check no gate is running
  (`pgrep -f gate.sh`), then remove it with `rm -rf` (the folder may hold an `owner` file, so `rmdir` fails). Release
  your own lock with `rm -rf` too, right after your gate ends, even when the gate fails.
- Known failures on the redesign/906 base itself (179f342f, checked in .scratch/906-redesign/base-ro): AdminUserDetailTabs,
  TradingLimitsSection, merchant-tier-panel-582, ticket-727-war-room-tooltips (copy and number strings). Not yours.
- After deleting a throwaway harness route, run `rm -rf frontend/.next/types` (or the whole `.next`) before the gate:
  typecheck reads stale `.next/types/validator.ts` and fails with TS2307 on the deleted page.
- design-md-tokens and ui-fixes fail in a slice worktree only because `<slice>-root/` lacks the root DESIGN.md and the
  backend folder. Fix once per worktree: `cp /Users/starbird/Workspace/Code/qic/.scratch/906-redesign/base-ro/DESIGN.md <slice>-root/`
  only. Do NOT link the backend into `<slice>-root/`: gate.sh then runs the whole backend suite (the shared backend has an
  untracked `target-memprobe/`). ui-fixes reads backend files, so it stays a known layout failure in slice worktrees
  (it passes in base-ro).
- Previews: start a dev server with `bunx next dev -p <your port>`, never `bun run dev`. The frontend `dev` script
  first runs `lsof -ti:3000 | xargs kill -9`, even with `-- -p <port>`, and so kills whoever holds :3000 (959-S6,
  22 Oct).
- Remove that backend symlink before running gate.sh (`rm <slice>-root/qictrader-backend-rs`). With it in place the gate runs
  the whole backend section against a base ref the backend repo lacks (968c saw "6131 changed files" and env-dependent
  cargo test failures). Recreate it only for a jest run that needs design-md-tokens or ui-fixes.

## Report
Write `/Users/starbird/Workspace/Code/qic/.scratch/<card>/reports/<slice>.md`: PR URL, head SHA, files changed, tests,
gate result, review verdicts, backend-blocked items with card numbers, board differences you could not close and why,
and any file you needed outside your ownership. Return at most eight lines.

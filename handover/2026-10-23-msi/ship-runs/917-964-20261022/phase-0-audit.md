# Phase 0 audit (917 + 964)

1. Error: L1 ran `budget phase PREFLIGHT` before the cold build and the preparation lock were clear, so a 22-minute phase stop was ticking during a wait that had nothing to do with ship work. A budget pause had to cover it.
   Cause: no rule says when PREFLIGHT opens relative to preparation and warm-up.
   Prevention: open the clock's first phase only after preparation is `ready` and the warm-up log shows EXIT=0; before that, leave the clock unstarted or paused with a run.log line.
2. Error: a wait loop `pgrep -f "<log name>"` matched its own command line and never exited.
   Cause: pgrep -f searches every process's full command line, including the waiter's.
   Prevention: wait on recorded PIDs (`kill -0 <pid>`), never on a pgrep of text the waiter itself contains.
3. Error: --verify-citations refused five file:line references in evidence.frontend_consumers (copied from the worker's facts with no citation). Fixed before the gate at no attempt cost.
   Cause: the worker cited the consumers by grep -n patterns that printed other line numbers.
   Prevention: none new; the existing path:line check caught it before Jev, as designed.
4. Observation: Jev objective 0.50 on a phase-0 evidence file the reviewer approved; same pattern as 819/821. No threshold change (user decision).
5. Preparation contention: another session held the preparation lock for about 30 minutes; the retry loop worked.

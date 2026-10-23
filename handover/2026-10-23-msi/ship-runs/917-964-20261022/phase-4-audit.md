# Phase 4 audit

1. The PUSH fast gate took about 27 minutes on this Mac against a 22-minute PUSH stop (5% of a 5 h L budget); a phase-overrun fired on a phase that went right. User authorized the push. Prevention (proposed, needs the user: thresholds are theirs): size the PUSH slice from the measured gate time, or skip the PUSH gate rerun when the tree equals a tree gated earlier in the run.
2. Cross-session messages to the parent timed out on a PreToolUse hook three times; the shared notes file and the user in this session were the fallback.
3. Jev overclaim probe fired on "main was never pushed"; L3 proved it with ls-remote. Prevention: cite every negative claim in the summary (citation contract rule already says so).

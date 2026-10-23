# Phase 3 audit (DRY, run 1 stopped and run 2)

1. Error (run 1): a shared cargo target-dir across worktrees made the DRY worker's test binary link another worktree's qictrader_backend_rs rlib; 8 DB tests failed against code without this change, the worker spent ~22 minutes on it, and DRY hit phase-overrun.
   Cause: cargo decides a workspace crate is fresh from source mtimes recorded with package-relative paths, so worktree Y's older files look fresh after worktree X builds into the same target.
   Prevention: every worktree builds into its own CARGO_TARGET_DIR (an APFS `cp -Rc` clone of a warm target costs seconds and almost no disk), set explicitly on every cargo command; touch the worktree's sources once after cloning. Never share a target-dir between worktrees of the same crate. (Parent removed the shared target-dir from qic/.cargo/config.toml.)
   Also: a test result that contradicts a just-passing gate is a stale-artifact suspect first; check the rlib's mtime against the worktree's last edit before debugging code.
2. Error (run 2): the parent asked to hold cargo for another session's gate while my DRY gate was running; DRY's slice was nearly spent.
   Prevention: close DRY on targeted re-proof plus check/clippy; the full fast gate belongs in PUSH on the committed tree (the integration gate), not twice.
3. Jev-only DRY gate refused on score alone; escalation to L3 worked as Speed rule 5 says, no attempt used.

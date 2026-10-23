# Phase 1 audit (910)
Errors
1. The shared cargo target (qic/.cargo/config.toml) could link another worktree's crate into 910's tests; the gate and two builder runs had to be re-run on a private clone (target-910), costing about 25 minutes.
2. Disk fell to 11-12 GB twice (two full targets on disk, then many test binaries in target-910).
3. The evidence writer found a missing AC test case (refused link then completion releases the lock) after the gate; fixed with one test.
4. A-910 took 30 minutes to get the shared names compiling (a cold check on a shared target under another session's load).
Causes
1. A process fix made earlier in the day (shared target) was wrong for multi-worktree use.
2. Two targets plus unpruned test binaries.
3. The builder brief listed the AC cases but the link case had two halves; the builder covered only the refusal half.
4. Shared-target lock contention.
Prevention
1. Fixed in the skill: one APFS clone per worktree with explicit CARGO_TARGET_DIR, touched sources; never a shared target.
2. Prune each run's clone after every full gate (skill rule 4).
3. Builder briefs that list AC cases must split multi-step cases into each observable half (refusal AND the follow-on behaviour).
4. Covered by 1.

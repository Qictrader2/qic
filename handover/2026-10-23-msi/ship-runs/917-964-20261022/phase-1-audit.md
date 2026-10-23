# Phase 1 audit

1. Builder C skipped the design package (did not open design-notes or boards) and cited a tail exit code; L1 caught it and sent it back. Prevention: builder briefs for frontend work list the design refinements (D-items) by name, not just the path to design-notes.
2. Builders A and B piped cargo through tail before echo EXIT, so the logs carried tail exit codes. The fast gate receipts were used as proof instead. Prevention: brief template shows the exact `( cmd 2>&1; echo "EXIT=$?" ) > log` form with no pipe.
3. Live check needed KYC_PROVIDER=didit with dummy values to boot; disk fell to 8.2 GB (shared target 29 GB of test binaries, 920 private target 23 GB). Asked the parent before deleting another run's cache.
4. Jev objective 0.52 on evidence L3 approved; same pattern.

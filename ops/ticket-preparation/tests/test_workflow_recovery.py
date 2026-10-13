import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from datetime import datetime, timedelta, timezone

spec = importlib.util.spec_from_file_location("preparation_recovery", Path(__file__).resolve().parents[1] / "qic_prepare.py")
q = importlib.util.module_from_spec(spec)
spec.loader.exec_module(q)


class RecoveryTests(unittest.TestCase):
    def evidence_card(self):
        return {"id": "abc", "idShort": 10, "name": "Proposal", "desc": "Owner-approved scope",
                "labels": [], "comments": [], "attachments": [],
                "checklists": [{"name": "Acceptance Criteria", "checkItems": []}]}

    def test_local_map_change_invalidates_cached_plan_or_blocker_without_a_card_edit(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            q.atomic_json(root / "ops/ticket-preparation/evidence-sources.json", {"10": [".scratch/map.md"]})
            path = root / ".scratch/map.md"
            path.parent.mkdir(exist_ok=True)
            path.write_text("27 redesigned pages")
            c = self.evidence_card()
            inputs = q.local_evidence(c, root)
            meta = {"card_id": "abc", "ticket_hash": q.digest(q.ticket_input(c)),
                    "watches": [], "local_evidence": inputs}
            self.assertEqual(q.current(meta, c, {}, local_inputs=inputs), (True, "current"))
            path.write_text("27 redesigned pages and complete baseline")
            changed = q.local_evidence(c, root)
            self.assertFalse(q.current(meta, c, {}, local_inputs=changed)[0])

    def test_newly_available_evidence_invalidates_the_missing_evidence_receipt(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            q.atomic_json(root / "ops/ticket-preparation/evidence-sources.json", {"10": ["map.md"]})
            before = q.local_evidence(self.evidence_card(), root)
            self.assertEqual(before, [{"path": "map.md", "sha256": None}])
            (root / "map.md").write_text("Baseline exists")
            self.assertNotEqual(q.local_evidence(self.evidence_card(), root), before)

    def design(self, root):
        (root / "design").mkdir()
        (root / "design/page.html").write_text("<html>Proposed design</html>")
        (root / "design/page.png").write_bytes(b"\x89PNG\r\n\x1a\nfixture")
        (root / "design/build.py").write_text("# local generator retained without executing")
        manifest = {"status": "ready", "review_state": "proposed", "inspected": True,
                    "states": ["Mobile and desktop"], "motion": ["Static under reduced motion"],
                    "questions": [], "files": ["design/page.html", "design/page.png", "design/build.py"]}
        return manifest

    def test_rejected_build_helper_is_repaired_without_changing_exports(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            manifest = self.design(root)
            q.atomic_json(root / "design-rejected-2.json", manifest)
            before = (root / "design/page.png").read_bytes()
            self.assertTrue(q.repair_design_manifest(root))
            self.assertEqual([p.name for p in q.validate_design(root)], ["page.html", "page.png"])
            self.assertEqual(q.read_json(root / "design.json")["internal_files"], ["design/build.py"])
            self.assertTrue((root / "design/build.py").is_file())
            self.assertEqual((root / "design/page.png").read_bytes(), before)
            self.assertFalse(q.repair_design_manifest(root))

    def test_unsupported_nonhelper_has_actionable_error_and_is_not_silently_removed(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            manifest = self.design(root)
            (root / "design/payload.exe").write_bytes(b"unsupported")
            manifest["files"] = ["design/page.html", "design/page.png", "design/payload.exe"]
            q.atomic_json(root / "design.json", manifest)
            self.assertFalse(q.repair_design_manifest(root))
            with self.assertRaisesRegex(q.PrepError, r"Unsupported design file: design/payload.exe"):
                q.validate_design(root)

    def test_internal_helper_cannot_escape_artifact_directory(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            manifest = self.design(root)
            manifest["files"].append("../secret.py")
            q.atomic_json(root / "design.json", manifest)
            with self.assertRaises(q.PrepError):
                q.repair_design_manifest(root)

    def test_preflight_report_cannot_replace_nightly_receipt(self):
        with tempfile.TemporaryDirectory() as tmp:
            original = q.STATE
            q.STATE = Path(tmp)
            try:
                nightly = q.finish_report(q.begin_report("scheduled"), [{"ticket": 10, "status": "ready"}])
                q.finish_report(q.begin_report("preflight"), [{"ticket": 20, "status": "current"}])
                self.assertEqual(q.read_json(q.STATE / "latest-nightly.json"), nightly)
                self.assertEqual(q.read_json(q.STATE / "last-scan.json"), nightly)
                self.assertEqual(q.read_json(q.STATE / "latest-preflight.json")["results"][0]["ticket"], 20)
                self.assertEqual(len(list((q.STATE / "runs").glob("*.json"))), 2)
            finally:
                q.STATE = original

    def test_transient_run_retries_after_cooldown_and_stops_after_three_attempts(self):
        now = datetime(2026, 10, 13, 0, 0, tzinfo=timezone.utc)
        previous = {"attempted_date": "2026-10-13", "attempts": 1}
        result = q.schedule_result(previous, now, True)
        self.assertNotIn("date", result)
        self.assertFalse(q.schedule_due(now + timedelta(minutes=14), result))
        self.assertTrue(q.schedule_due(now + timedelta(minutes=15), result))
        result["attempts"] = 3
        self.assertFalse(q.schedule_due(now + timedelta(hours=1), result))

    def test_finished_blocked_scan_does_not_repeat_models_all_day(self):
        now = datetime(2026, 10, 13, 0, 0, tzinfo=timezone.utc)
        result = q.schedule_result({"attempted_date": "2026-10-13", "attempts": 1}, now, False)
        self.assertEqual(result["date"], "2026-10-13")
        self.assertFalse(q.schedule_due(now + timedelta(hours=2), result))
        self.assertTrue(q.schedule_due(now + timedelta(days=1), result))

    def test_proposal_plan_preserves_approval_and_cannot_claim_ordinary_ready_status(self):
        body = "# Proposed pack\n"
        metadata = {"version": 1, "status": "proposal-ready", "card_id": "abc", "ticket_hash": "hash",
                    "watches": [{}], "body_sha256": q.digest(body.encode()), "frontend": False,
                    "implementation_gates": ["Christian approves the versioned pack"]}
        parsed, _ = q.parse_plan(q.plan_bytes(metadata, body))
        self.assertEqual(q.implementation_gates(parsed), ["Christian approves the versioned pack"])
        metadata["implementation_gates"] = []
        with self.assertRaisesRegex(q.PrepError, "implementation approval gate"):
            q.parse_plan(q.plan_bytes(metadata, body))


if __name__ == "__main__":
    unittest.main()

import copy
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from datetime import datetime, timezone

spec = importlib.util.spec_from_file_location("qic_prepare", Path(__file__).resolve().parents[1] / "qic_prepare.py")
q = importlib.util.module_from_spec(spec)
spec.loader.exec_module(q)


def card():
    return {"id": "abc", "idShort": 10, "name": "Show the confirmed amount", "desc": "Expected behaviour: confirmed amount appears",
            "labels": [], "comments": [], "attachments": [],
            "checklists": [{"name": "Acceptance Criteria", "checkItems": [{"id": "ac1", "name": "Confirmed amount appears", "pos": 1, "state": "incomplete"}]}]}


def investigation():
    return {"diagnosis": "Reader omits the confirmed value", "expected_behaviour": "Confirmed value appears", "scope": "Amount reader",
            "implementation": "Use the confirmed amount in the reader", "severity": "logic", "frontend": False,
            "estimated_lines": 20, "estimated_files": 1, "risks": [], "watch": {"backend": ["reader.txt"]},
            "questions": [], "acceptance": [{"criterion_id": "ac1", "cases": [{"setup": "Confirm 10 USDT", "expected": "10 USDT", "evidence": "Real API response"}]}]}


class ContractTests(unittest.TestCase):
    def test_weekly_threshold_is_strictly_above_ninety(self):
        for percent in (0, 89.9, 90):
            route = q.route_for_usage({"seven_day": {"utilization": percent}})
            self.assertEqual((route["provider"], route["model"], route["effort"]), ("claude", q.MODEL, "high"))
        for percent in (90.01, 98, 100):
            route = q.route_for_usage({"seven_day": {"utilization": percent}})
            self.assertEqual((route["provider"], route["model"], route["effort"]), ("codex", "gpt-6.1-sol", "high"))

    def test_invalid_or_missing_usage_defers_instead_of_spending(self):
        readings = [None, {}, {"seven_day": None}]
        readings.extend({"seven_day": {"utilization": value}} for value in (None, True, "98", -1, 101, float("nan"), float("inf")))
        for usage in readings:
            with self.assertRaises(q.WorkerCapacityError):
                q.route_for_usage(usage)

    def test_codex_bypasses_claude_cooldown_and_claude_still_obeys_it(self):
        now = datetime(2026, 10, 9, tzinfo=timezone.utc)
        capacity = {"retry_at": "2026-10-10T00:00:00+00:00", "reason": "Claude quota"}
        self.assertEqual(q.route_for_worker({"seven_day": {"utilization": 98}}, capacity, now)["provider"], "codex")
        with self.assertRaises(q.WorkerCapacityError):
            q.route_for_worker({"seven_day": {"utilization": 90}}, capacity, now)

    def test_codex_receipt_requires_success_and_exact_model_and_effort(self):
        transcript = "OpenAI Codex\n--------\nmodel: gpt-6.1-sol\nprovider: openai\napproval: never\nreasoning effort: high\n--------\nuser\nTask\n"
        receipt = q.codex_receipt(transcript, "Done", 0)
        self.assertEqual(receipt["modelUsage"][q.CODEX_MODEL]["effort"], "high")
        for log, final, code in ((transcript, "Done", 1), (transcript, "", 0),
                                 (transcript.replace("gpt-6.1-sol", "gpt-6-sol"), "Done", 0),
                                 (transcript.replace("effort: high", "effort: low"), "Done", 0),
                                 ("user\n" + transcript, "Done", 0)):
            with self.assertRaises(q.PrepError):
                q.codex_receipt(log, final, code)

    def test_codex_draft_resumes_only_with_verified_high_effort_receipt(self):
        with tempfile.TemporaryDirectory() as d:
            directory = Path(d)
            path = directory / "ticket-planner-result.json"
            receipt = {"type": "result", "subtype": "success", "is_error": False,
                       "modelUsage": {q.CODEX_MODEL: {"canonicalModel": q.CODEX_MODEL, "provider": "openai", "effort": "high", "runtimeEvidence": "CLI header"}}}
            path.write_text(json.dumps(receipt))
            self.assertTrue(q.worker_finished("ticket-planner", directory))
            receipt["modelUsage"][q.CODEX_MODEL]["effort"] = "low"
            path.write_text(json.dumps(receipt))
            self.assertFalse(q.worker_finished("ticket-planner", directory))
            path.write_text(json.dumps({"type": "running", "model": q.CODEX_MODEL}))
            self.assertFalse(q.worker_finished("ticket-planner", directory))

    def test_render_requests_reject_traversal_symlinks_and_invalid_viewports(self):
        with tempfile.TemporaryDirectory() as d:
            directory = Path(d)
            (directory / "design.html").write_text("<p>Poppins</p>")
            request = {"html": "design.html", "prefix": "desktop", "width": 1440, "height": 1100}
            path = directory / "design-render-requests.json"
            path.write_text(json.dumps({"renders": [request]}))
            self.assertEqual(q.design_render_requests(directory)[0]["html_sha256"], q.digest((directory / "design.html").read_bytes()))
            (directory / "linked.html").symlink_to(directory / "design.html")
            for change in ({"html": "../outside.html"}, {"prefix": "../outside"}, {"html": "linked.html"},
                           {"width": True}, {"height": 3001}):
                path.write_text(json.dumps({"renders": [{**request, **change}]}))
                with self.assertRaises(q.PrepError):
                    q.design_render_requests(directory)
            path.write_text(json.dumps({"renders": [request, request]}))
            with self.assertRaises(q.PrepError):
                q.design_render_requests(directory)

    def test_claude_quota_reset_uses_dubai_even_from_utc(self):
        now = datetime(2026, 10, 7, 14, 0, tzinfo=timezone.utc)
        retry = q.quota_retry_at("You've hit your session limit · resets 8:40pm (Asia/Dubai)", now)
        self.assertEqual(retry.isoformat(), "2026-10-07T20:40:00+04:00")

    def test_unparseable_quota_defers_an_hour_without_guessing_a_daily_reset(self):
        now = datetime(2026, 10, 7, 14, 0, tzinfo=timezone.utc)
        self.assertEqual(q.quota_retry_at("Rate limited", now).isoformat(), "2026-10-07T15:00:00+00:00")

    def test_interrupted_or_wrong_model_draft_is_not_a_completed_worker(self):
        with tempfile.TemporaryDirectory() as d:
            directory = Path(d)
            result = directory / "design-expert-result.json"
            result.write_text("")
            self.assertFalse(q.worker_finished("design-expert", directory))
            receipt = {"type": "result", "subtype": "success", "is_error": False,
                       "modelUsage": {"other-model": {}}}
            result.write_text(json.dumps(receipt))
            self.assertFalse(q.worker_finished("design-expert", directory))
            receipt["modelUsage"] = {q.MODEL: {"canonicalModel": q.MODEL}}
            result.write_text(json.dumps(receipt))
            self.assertTrue(q.worker_finished("design-expert", directory))

    def test_schedule_uses_dubai_time_and_runs_once_per_day(self):
        self.assertFalse(q.schedule_due(datetime(2026, 10, 7, 22, 59, tzinfo=timezone.utc), {}))
        self.assertTrue(q.schedule_due(datetime(2026, 10, 7, 23, 0, tzinfo=timezone.utc), {}))
        self.assertFalse(q.schedule_due(datetime(2026, 10, 7, 23, 1, tzinfo=timezone.utc), {"date": "2026-10-08"}))

    def test_schedule_catches_up_after_machine_wakes(self):
        self.assertTrue(q.schedule_due(datetime(2026, 10, 8, 7, 0, tzinfo=timezone.utc), {"date": "2026-10-07"}))

    def test_plan_rejects_body_tampering(self):
        body = "# Complete plan\n"
        metadata = {"version": 1, "status": "ready", "body_sha256": q.digest(body.encode()), "watches": [{}], "card_id": "abc", "ticket_hash": "hash", "frontend": False}
        data = q.plan_bytes(metadata, body)
        self.assertEqual(q.parse_plan(data)[1], body)
        with self.assertRaises(q.PrepError):
            q.parse_plan(data.replace(b"Complete plan", b"Changed plan"))

    def test_frontend_plan_requires_exports(self):
        body = "# Plan\n"
        meta = {"version": 1, "status": "ready", "body_sha256": q.digest(body.encode()), "watches": [{}], "card_id": "abc", "ticket_hash": "hash", "frontend": True}
        with self.assertRaises(q.PrepError):
            q.parse_plan(q.plan_bytes(meta, body))

    def test_every_acceptance_criterion_is_required(self):
        p = investigation()
        p["acceptance"] = []
        with self.assertRaises(q.PrepError):
            q.validate_investigation(p, card())

    def test_duplicate_acceptance_mapping_is_rejected(self):
        p = investigation()
        p["acceptance"] *= 2
        with self.assertRaises(q.PrepError):
            q.validate_investigation(p, card())

    def test_plan_numbering_follows_ticket_even_when_planner_reorders_cases(self):
        c = card()
        c["checklists"][0]["checkItems"].append({"id": "ac2", "name": "Pending amount is labelled", "pos": 2})
        p = investigation()
        p["acceptance"].insert(0, {"criterion_id": "ac2", "cases": [{"setup": "Pending deposit", "expected": "Pending", "evidence": "UI assertion"}]})
        body = q.render_body(c, q.validate_investigation(p, c), [])
        self.assertLess(body.index("AC1: Confirmed amount appears"), body.index("AC2: Pending amount is labelled"))
        self.assertIn("AC2: Pending amount is labelled\n- Setup: Pending deposit", body)

    def test_financial_risk_cannot_use_light_track(self):
        p = investigation(); p["risks"] = ["ledger"]
        self.assertEqual(q.validate_investigation(p, card())["track"], "FULL")

    def test_small_low_risk_scope_uses_light_track(self):
        self.assertEqual(q.validate_investigation(investigation(), card())["track"], "LIGHT")

    def test_frontend_label_cannot_skip_design(self):
        c = card(); c["labels"] = [{"name": "frontend"}]
        self.assertTrue(q.validate_investigation(investigation(), c)["frontend"])

    def test_read_only_frontend_dependency_is_not_itself_frontend_work(self):
        p = investigation(); p["watch"] = {"frontend": ["src"]}
        self.assertFalse(q.validate_investigation(p, card())["frontend"])

    def test_managed_attachments_and_checked_boxes_do_not_invalidate_requirements(self):
        a = card(); b = copy.deepcopy(a)
        b["attachments"] = [{"id": "p", "name": "PLAN.md"}, {"id": "d", "name": "qic-prep-10-design.png"}]
        b["checklists"][0]["checkItems"][0]["state"] = "complete"
        self.assertEqual(q.digest(q.ticket_input(a)), q.digest(q.ticket_input(b)))

    def test_new_owner_decision_invalidates_plan(self):
        a = card(); b = copy.deepcopy(a)
        b["comments"] = [{"id": "new", "data": {"text": "Round down to cents"}}]
        self.assertNotEqual(q.digest(q.ticket_input(a)), q.digest(q.ticket_input(b)))

    def test_user_design_attachment_is_an_input(self):
        a = card(); b = copy.deepcopy(a)
        b["attachments"] = [{"id": "user", "name": "DESIGN.png", "url": "https://trello.com/file", "bytes": 100}]
        self.assertNotEqual(q.digest(q.ticket_input(a)), q.digest(q.ticket_input(b)))

    def test_path_traversal_is_rejected(self):
        for path in ("../secret", "/etc/passwd", "a/../../secret", "--option"):
            with self.assertRaises(q.PrepError): q.safe_relative(path)

    def test_design_without_editable_source_is_not_ready(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d)
            (p / "design.json").write_text(json.dumps({"status": "ready", "review_state": "proposed", "inspected": True,
                "motion": ["No motion"], "states": ["Desktop"], "questions": [], "files": []}))
            with self.assertRaises(q.PrepError): q.validate_design(p)

    def test_blocked_design_cannot_publish(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d)
            (p / "design.json").write_text(json.dumps({"status": "ready", "review_state": "proposed", "questions": ["Which default?"]}))
            with self.assertRaises(q.PrepError): q.validate_design(p)


class GitFreshnessTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.repo = Path(self.temp.name)
        subprocess.run(["git", "init", "-q", str(self.repo)], check=True)
        q.git(self.repo, "config", "user.name", "Test")
        q.git(self.repo, "config", "user.email", "test@example.invalid")
        (self.repo / "reader.txt").write_text("confirmed\n")
        (self.repo / "other.txt").write_text("unrelated\n")
        self.commit()
        self.base = q.git(self.repo, "rev-parse", "HEAD")
        self.repos = {"backend": self.repo}
        self.pins = {"backend": self.base}
        self.meta = {"card_id": "abc", "ticket_hash": q.digest(q.ticket_input(card())),
                     "watches": q.watches_for({"backend": ["reader.txt"]}, self.pins, self.repos)}

    def tearDown(self):
        self.temp.cleanup()

    def commit(self):
        q.git(self.repo, "add", ".")
        q.git(self.repo, "commit", "-qm", "fixture")

    def draft_fixture(self):
        (self.repo / "DESIGN.md").write_text("Poppins and semantic tokens\n")
        self.commit()
        pin = q.git(self.repo, "rev-parse", "HEAD")
        temp = tempfile.TemporaryDirectory(); self.addCleanup(temp.cleanup)
        prior = Path(temp.name) / "prior"; prior.mkdir()
        destination = Path(temp.name) / "current"; destination.mkdir()
        c = card(); c["url"] = "https://trello.com/c/fixture"
        q.atomic_json(prior / "ticket.json", {**q.ticket_input(c), "idShort": c["idShort"], "url": c["url"]})
        q.atomic_json(prior / "context.json", {"pins": {"root": pin, "backend": pin}})
        q.atomic_json(prior / "investigation.json", investigation())
        q.atomic_json(prior / "ticket-planner-result.json", {"type": "result", "subtype": "success", "is_error": False, "modelUsage": {q.MODEL: {}}})
        (prior / "design").mkdir(); (prior / "design/example.html").write_text("<p>Preserved draft</p>")
        return destination, c, {"root": self.repo, "backend": self.repo}

    def test_unrelated_commit_resumes_completed_planning_without_another_worker(self):
        directory, c, repos = self.draft_fixture()
        (self.repo / "other.txt").write_text("updated unrelated source"); self.commit()
        q.resume_draft(directory, c, {"root": "HEAD", "backend": "HEAD"}, repos)
        self.assertTrue(q.worker_finished("ticket-planner", directory))
        self.assertFalse(q.worker_finished("design-expert", directory))
        self.assertEqual((directory / "design/example.html").read_text(), "<p>Preserved draft</p>")

    def test_changed_dependency_preserves_designs_but_requires_fresh_planner_verdict(self):
        directory, c, repos = self.draft_fixture()
        (self.repo / "reader.txt").write_text("changed reader behavior"); self.commit()
        q.resume_draft(directory, c, {"root": "HEAD", "backend": "HEAD"}, repos)
        self.assertFalse(q.worker_finished("ticket-planner", directory))
        self.assertFalse((directory / "investigation.json").exists())
        self.assertIn("changed reader behavior", (directory / "source-delta.txt").read_text())
        self.assertTrue((directory / "design/example.html").exists())

    def test_changed_requirements_do_not_reuse_previous_drafts(self):
        directory, c, repos = self.draft_fixture()
        c["desc"] = "New expected behavior"
        q.resume_draft(directory, c, {"root": "HEAD", "backend": "HEAD"}, repos)
        self.assertFalse((directory / "resumed-from.json").exists())

    def test_rejected_manifest_resumes_across_unrelated_source_commit_without_redesign(self):
        directory, c, repos = self.draft_fixture()
        prior = directory.parent / "prior"
        (prior / "design/board.png").write_bytes(b"\x89PNG\r\n\x1a\nfixture")
        (prior / "design/build.py").write_text("# retained generator")
        q.atomic_json(prior / "design-rejected-2.json", {
            "status": "ready", "review_state": "proposed", "inspected": True, "questions": [],
            "states": ["Desktop"], "motion": ["Static"],
            "files": ["design/example.html", "design/board.png", "design/build.py"]})
        q.atomic_json(prior / "design-expert-result.json", {"type": "result", "subtype": "success",
            "is_error": False, "modelUsage": {q.MODEL: {}}})
        (self.repo / "other.txt").write_text("unrelated change")
        self.commit()
        q.resume_draft(directory, c, {"root": "HEAD", "backend": "HEAD"}, repos)
        self.assertTrue(q.worker_finished("design-expert", directory))
        self.assertEqual([p.name for p in q.validate_design(directory)], ["example.html", "board.png"])
        self.assertTrue((prior / "design-rejected-2.json").exists())
        self.assertTrue((directory / "design/build.py").exists())

    def test_unrelated_commit_keeps_plan_current(self):
        (self.repo / "other.txt").write_text("changed\n"); self.commit()
        self.assertEqual(q.current(self.meta, card(), {"backend": "HEAD"}, self.repos), (True, "current"))

    def test_changed_dependency_requires_preparation(self):
        (self.repo / "reader.txt").write_text("changed\n"); self.commit()
        self.assertFalse(q.current(self.meta, card(), {"backend": "HEAD"}, self.repos)[0])

    def test_changed_expected_behaviour_requires_preparation(self):
        c = card(); c["desc"] = "Expected behaviour: show 20 USDT"
        self.assertFalse(q.current(self.meta, c, self.pins, self.repos)[0])

    def test_other_ticket_cannot_reuse_plan(self):
        c = card(); c["id"] = "different"
        self.assertFalse(q.current(self.meta, c, self.pins, self.repos)[0])

    def test_nonexistent_path_requires_existing_parent(self):
        with self.assertRaises(q.PrepError): q.watch_hash(self.repo, "HEAD", "missing.txt")

    def test_new_file_watch_expands_to_tracked_parent(self):
        watches = q.watches_for({"backend": ["missing.txt"]}, self.pins, self.repos)
        self.assertEqual(watches[0]["path"], ".")
        self.assertEqual(watches[0]["requested_path"], "missing.txt")

    def test_deleted_dependency_invalidates_without_crashing(self):
        (self.repo / "reader.txt").unlink(); self.commit()
        self.assertFalse(q.current(self.meta, card(), {"backend": "HEAD"}, self.repos)[0])


if __name__ == "__main__":
    unittest.main()

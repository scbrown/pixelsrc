#!/usr/bin/env python3
"""Offline dispatch and publishing-boundary controls; never call GitHub or crates.io."""
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class DispatchControls(unittest.TestCase):
    def setUp(self):
        self.folder = tempfile.TemporaryDirectory()
        self.addCleanup(self.folder.cleanup)
        self.temp = Path(self.folder.name)
        self.calls = self.temp / "calls.jsonl"
        command = self.temp / "gh"
        command.write_text(
            "#!" + sys.executable + "\n"
            "import json,os,sys\n"
            "with open(os.environ['TEST_CALLS'],'a') as f:\n"
            " f.write(json.dumps(sys.argv[1:])+'\\n')\n"
            "sys.exit(int(os.environ.get('TEST_EXIT','0')))\n"
        )
        command.chmod(0o700)
        self.env = {**os.environ, "GITHUB_REPOSITORY": "scbrown/pixelsrc",
                    "TEST_CALLS": str(self.calls), "PATH": str(self.temp) + os.pathsep + os.defpath}

    def run_dispatch(self, tag="v1.2.3", **overrides):
        env = {**self.env, "RELEASE_TAG": tag, **overrides}
        return subprocess.run(["bash", str(ROOT / "scripts/dispatch-crates.sh")],
                              env=env, capture_output=True, text=True, timeout=10)

    def recorded(self):
        return [json.loads(line) for line in self.calls.read_text().splitlines()] if self.calls.exists() else []

    def test_exact_workflow_repository_ref_and_tag_are_dispatched_once(self):
        result = self.run_dispatch()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.recorded(), [["workflow", "run", "crates.yml", "--repo",
                         "scbrown/pixelsrc", "--ref", "main", "-f", "tag=v1.2.3",
                         "-f", "dry_run=false"]])

    def test_prerelease_and_build_metadata_are_valid_tags(self):
        for tag in ["v0.3.0-rc.1", "v0.3.0+build.1"]:
            with self.subTest(tag=tag):
                self.assertEqual(self.run_dispatch(tag).returncode, 0)
        self.assertEqual(len(self.recorded()), 2)

    def test_fork_or_unknown_repository_refuses_before_dispatch(self):
        for repo in ["someone/pixelsrc", "", "scbrown/other"]:
            with self.subTest(repo=repo):
                self.assertEqual(self.run_dispatch(GITHUB_REPOSITORY=repo).returncode, 2)
        self.assertEqual(self.recorded(), [])

    def test_missing_branch_malformed_and_shell_tags_refuse_without_a_call(self):
        for tag in ["", "main", "v1.2", "../v1.2.3", "v1.2.3/branch",
                    "v1.2.3\nv4.5.6", "v1.2.3; echo bad", "$(echo bad)"]:
            with self.subTest(tag=tag):
                self.assertEqual(self.run_dispatch(tag).returncode, 2)
        self.assertEqual(self.recorded(), [])

    def test_dispatch_failure_is_not_swallowed_or_retried(self):
        result = self.run_dispatch(TEST_EXIT="7")
        self.assertEqual(result.returncode, 7)
        self.assertEqual(len(self.recorded()), 1)


class WorkflowBindings(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.caller = (ROOT / ".github/workflows/release-plz.yml").read_text()
        cls.callee = (ROOT / ".github/workflows/crates.yml").read_text()

    def test_caller_uses_the_tested_dispatcher_with_minimal_permissions(self):
        job = self.caller.split("\n  crates:\n", 1)[1]
        self.assertNotIn("uses: ./.github/workflows/crates.yml", self.caller)
        self.assertIn("actions: write", job)
        self.assertNotIn("id-token: write", job)
        self.assertIn("GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}", job)
        self.assertIn("RELEASE_TAG: ${{ needs.release-plz.outputs.tag }}", job)
        self.assertIn("run: bash scripts/dispatch-crates.sh", job)
        self.assertIn("if: needs.release-plz.outputs.releases_created == 'true'", job)

    def test_callee_is_not_reusable_and_requires_a_dispatch_tag(self):
        self.assertNotIn("workflow_call:", self.callee)
        self.assertRegex(self.callee, r"workflow_dispatch:\s+inputs:\s+tag:\s+description:[^\n]+\s+required: true\s+type: string")
        self.assertIn("ref: refs/tags/${{ steps.release-tag.outputs.tag }}", self.callee)

    def test_dry_run_cannot_authenticate_or_publish(self):
        self.assertRegex(self.callee, r"name: Publish \(dry run\)\s+if: \$\{\{ inputs.dry_run == true \}\}\s+run: cargo publish --dry-run")
        for name in ["Authenticate with crates.io", "Publish to crates.io"]:
            with self.subTest(step=name):
                self.assertRegex(self.callee, re.escape("- name: " + name) + r"\s+if: \$\{\{ inputs.dry_run != true \}\}")

    def test_real_callee_validation_script_refuses_bad_tags(self):
        match = re.search(r"      - name: Validate release tag\n.*?        run: \|\n(.*?)      - uses:", self.callee, re.S)
        self.assertIsNotNone(match)
        script = "\n".join(line[10:] for line in match[1].splitlines())
        with tempfile.TemporaryDirectory() as folder:
            output = Path(folder) / "output"
            for tag, expected in [("v1.2.3", 0), ("", 2), ("main", 2), ("v1.2.3/branch", 2)]:
                with self.subTest(tag=tag):
                    result = subprocess.run(["bash", "-e", "-c", script], capture_output=True,
                                            text=True, timeout=10,
                                            env={**os.environ, "RELEASE_TAG": tag, "GITHUB_OUTPUT": str(output),
                                                 "GITHUB_REPOSITORY": "scbrown/pixelsrc",
                                                 "GITHUB_WORKFLOW_REF": "scbrown/pixelsrc/.github/workflows/crates.yml@refs/heads/main"})
                    self.assertEqual(result.returncode, expected, result.stderr)
            self.assertEqual(output.read_text(), "tag=v1.2.3\n")

    def test_reusable_caller_and_fork_identity_refuse_before_authentication(self):
        match = re.search(r"      - name: Validate release tag\n.*?        run: \|\n(.*?)      - uses:", self.callee, re.S)
        script = "\n".join(line[10:] for line in match[1].splitlines())
        with tempfile.TemporaryDirectory() as folder:
            output = Path(folder) / "output"
            for repo, workflow in [
                ("scbrown/pixelsrc", "scbrown/pixelsrc/.github/workflows/release-plz.yml@refs/heads/main"),
                ("someone/pixelsrc", "someone/pixelsrc/.github/workflows/crates.yml@refs/heads/main"),
                ("scbrown/pixelsrc", ""),
            ]:
                with self.subTest(repository=repo, workflow=workflow):
                    result = subprocess.run(["bash", "-e", "-c", script], capture_output=True,
                                            text=True, timeout=10, env={**os.environ,
                                            "RELEASE_TAG": "v1.2.3", "GITHUB_OUTPUT": str(output),
                                            "GITHUB_REPOSITORY": repo, "GITHUB_WORKFLOW_REF": workflow})
                    self.assertEqual(result.returncode, 2, result.stderr)
            self.assertFalse(output.exists())

    def test_controls_are_in_normal_pull_request_ci(self):
        ci = (ROOT / ".github/workflows/ci.yml").read_text()
        self.assertIn("run: python3 scripts/test-crates-dispatch.py", ci)


if __name__ == "__main__":
    unittest.main(verbosity=2)

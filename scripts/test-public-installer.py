"""Exercise the actual embedded installer with local release fixtures only."""
import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
SOURCE = (ROOT / "install.sh").read_text().split("<<'PY'\n", 1)[1].rsplit("\nPY", 1)[0]


def archive(payload=b'#!/bin/sh\nprintf "pxl 0.3.1\\n"\n', name="pxl", kind=None, extra=False):
    output = io.BytesIO()
    with tarfile.open(fileobj=output, mode="w:gz") as package:
        member = tarfile.TarInfo(name)
        member.size = len(payload)
        if kind:
            member.type = kind
            member.linkname = "/tmp/outside"
            member.size = 0
        package.addfile(member, io.BytesIO(payload))
        if extra:
            package.addfile(tarfile.TarInfo("unexpected"))
    return output.getvalue()


class InstallerTests(unittest.TestCase):
    def setUp(self):
        self.scratch = tempfile.TemporaryDirectory()
        self.addCleanup(self.scratch.cleanup)
        self.home = Path(self.scratch.name).resolve()
        self.env = patch.dict(os.environ, {"HOME": str(self.home)}, clear=False)
        self.env.start()
        self.addCleanup(self.env.stop)
        self.ns = {"__name__": "installer_fixture"}
        exec(compile(SOURCE, "install.sh", "exec"), self.ns)
        self.ns["target"] = lambda: "x86_64-unknown-linux-gnu"
        self.which = patch("shutil.which", return_value=None)
        self.which.start()
        self.addCleanup(self.which.stop)
        self.name = "pxl-v0.3.1-x86_64-unknown-linux-gnu.tar.gz"
        self.base = "https://github.com/scbrown/pixelsrc/releases/download/v0.3.1/"
        self.info = {"tag_name": "v0.3.1", "draft": False, "prerelease": False,
                     "assets": [{"name": n, "browser_download_url": self.base + n}
                                for n in (self.name, "SHA256SUMS.txt")]}
        self.payload = archive()
        self.calls = []
        self.ns["fetch"] = self.fetch

    def fetch(self, url, limit):
        self.calls.append(url)
        if url.startswith("https://api.github.com/repos/scbrown/pixelsrc/releases/"):
            return json.dumps(self.info).encode()
        if url == self.base + self.name:
            return self.payload
        if url == self.base + "SHA256SUMS.txt":
            return (hashlib.sha256(self.payload).hexdigest() + "  " + self.name).encode()
        self.fail(f"unexpected download: {url}")

    def install(self, *args):
        return self.ns["main"](list(args))

    def test_default_real_command_and_nonoverwrite(self):
        self.install()
        link = self.home / ".local/bin/pxl"
        self.assertTrue(link.is_symlink())
        self.assertEqual(subprocess.check_output([str(link), "--version"], text=True).strip(),
                         "pxl 0.3.1")
        before = link.resolve().read_bytes()
        with self.assertRaisesRegex(ValueError, "prefix already exists"):
            self.install()
        self.assertEqual(link.resolve().read_bytes(), before)

    def test_custom_prefix_and_version(self):
        prefix = self.home / "isolated"
        self.install("--version", "0.3.1", "--prefix", str(prefix))
        self.assertTrue((prefix / "pxl").is_file())
        self.assertFalse((self.home / ".local/bin").exists())
        self.assertTrue(self.calls[0].endswith("tags/v0.3.1"))

    def test_dry_run_only_metadata_no_files(self):
        self.install("--dry-run")
        self.assertEqual(len(self.calls), 1)
        self.assertEqual(list(self.home.iterdir()), [])

    def test_existing_command_and_dangling_link_refused(self):
        link = self.home / ".local/bin/pxl"
        link.parent.mkdir(parents=True)
        link.symlink_to(self.home / "absent")
        with self.assertRaisesRegex(ValueError, "already exists"):
            self.install()
        self.assertTrue(link.is_symlink())
        self.assertEqual(len(self.calls), 1)

    def test_path_managed_install_refused(self):
        with patch("shutil.which", return_value="/usr/bin/pxl"):
            with self.assertRaisesRegex(ValueError, "package manager"):
                self.install()
        self.assertEqual(list(self.home.iterdir()), [])

    def test_wrong_binary_version_cleans_only_candidate(self):
        self.payload = archive(b'#!/bin/sh\nprintf "pxl 0.0.0\\n"\n')
        sentinel = self.home / "keep"
        sentinel.write_text("retained")
        with self.assertRaisesRegex(ValueError, "unexpected version"):
            self.install()
        self.assertFalse((self.home / ".local/share/pixelsrc/v0.3.1").exists())
        self.assertFalse((self.home / ".local/bin/pxl").exists())
        self.assertEqual(sentinel.read_text(), "retained")

    def test_child_failure_cleans_candidate(self):
        self.payload = archive(b"#!/bin/sh\nexit 4\n")
        with self.assertRaises(subprocess.CalledProcessError):
            self.install()
        self.assertFalse((self.home / ".local/share/pixelsrc/v0.3.1").exists())

    def test_credentials_not_forwarded_to_release_binary(self):
        self.payload = archive(b'#!/bin/sh\n[ -z "${GH_TOKEN+x}${GITHUB_TOKEN+x}${QUIPU_AUTH_TOKEN+x}" ] || exit 9\nprintf "pxl 0.3.1\\n"\n')
        with patch.dict(os.environ, {"GH_TOKEN": "fixture", "GITHUB_TOKEN": "fixture",
                                    "QUIPU_AUTH_TOKEN": "fixture"}):
            self.install()

    def test_invalid_versions_before_network(self):
        for version in ("v0.3.1", "1.2.3/../latest", "1.2", "1.2.3-rc1"):
            with self.subTest(version=version), self.assertRaises(ValueError):
                self.install("--version", version)
        self.assertEqual(self.calls, [])

    def test_release_identity_draft_and_prerelease_refused(self):
        for change in ({"tag_name": "v0.3.1-rc1"}, {"draft": True}, {"prerelease": True},
                       {"tag_name": "0.3.1"}):
            saved = self.info.copy()
            self.info.update(change)
            with self.subTest(change=change), self.assertRaises(ValueError):
                self.install()
            self.info = saved
        self.assertEqual(list(self.home.iterdir()), [])

    def test_wrong_tag_for_pin_refused(self):
        with self.assertRaises(ValueError):
            self.install("--version", "9.9.9")

    def test_asset_identity_and_duplicates_refused(self):
        for assets in ([], self.info["assets"] * 2,
                       [{"name": self.name, "browser_download_url": "https://example.com/payload"}]):
            with self.subTest(assets=assets):
                self.info["assets"] = assets
                with self.assertRaises(ValueError):
                    self.install()
        self.assertEqual(list(self.home.iterdir()), [])

    def test_checksum_and_duplicate_lines_refused(self):
        checksum = hashlib.sha256(self.payload).hexdigest()
        for sums in (b"", ("0" * 64 + "  " + self.name).encode(),
                     ((checksum + "  " + self.name + "\n") * 2).encode()):
            with self.subTest(sums=sums), self.assertRaises(ValueError):
                self.ns["checked_binary"](self.payload, sums, self.name)

    def test_archive_traversal_links_extra_and_wrong_name_refused(self):
        for body in (archive(name="../pxl"), archive(name="/pxl"), archive(name="other"),
                     archive(kind=tarfile.SYMTYPE), archive(kind=tarfile.LNKTYPE),
                     archive(extra=True), archive(payload=b"")):
            sums = (hashlib.sha256(body).hexdigest() + "  " + self.name).encode()
            with self.subTest(body=hashlib.sha256(body).hexdigest()), self.assertRaises(ValueError):
                self.ns["checked_binary"](body, sums, self.name)
        self.assertFalse((self.home.parent / "pxl").exists())

    def test_decompression_is_bounded_before_tar_parsing(self):
        import gzip
        body = gzip.compress(b"x" * (1024 * 1024 + 4096))
        self.ns["MAX_BINARY"] = 1
        sums = (hashlib.sha256(body).hexdigest() + "  " + self.name).encode()
        with self.assertRaisesRegex(ValueError, "decompressed"):
            self.ns["checked_binary"](body, sums, self.name)

    def test_redirect_boundary(self):
        for url in ("http://github.com/file", "https://github.com.example.org/file",
                    "https://user:pass@github.com/file", "https://github.com:444/file"):
            with self.subTest(url=url), self.assertRaises(ValueError):
                self.ns["check_url"](url)
        for host in ("github.com", "api.github.com", "release-assets.githubusercontent.com"):
            self.ns["check_url"](f"https://{host}/file")

    def test_actual_fetch_enforces_size_and_sends_no_authorization(self):
        ns = {"__name__": "fixture"}
        exec(compile(SOURCE, "install.sh", "exec"), ns)
        class Response(io.BytesIO):
            def geturl(self):
                return "https://github.com/scbrown/pixelsrc/releases/download/v0.3.1/file"
        class Opener:
            def open(inner, request, timeout):
                self.assertNotIn("Authorization", request.headers)
                self.assertEqual(timeout, 30)
                return Response(b"12345")
        with patch("urllib.request.build_opener", return_value=Opener()):
            self.assertEqual(ns["fetch"]("https://github.com/file", 5), b"12345")
            with self.assertRaisesRegex(ValueError, "size limit"):
                ns["fetch"]("https://github.com/file", 4)

    def test_late_command_collision_preserves_existing_file(self):
        link = self.home / ".local/bin/pxl"
        original = Path.symlink_to
        def collide(path, target, *args, **kwargs):
            path.write_text("another installation")
            return original(path, target, *args, **kwargs)
        with patch.object(Path, "symlink_to", collide):
            with self.assertRaises(FileExistsError):
                self.install()
        self.assertEqual(link.read_text(), "another installation")
        self.assertFalse((self.home / ".local/share/pixelsrc/v0.3.1").exists())

    def test_four_targets_and_musl_refusal(self):
        # Reload the actual target function instead of testing the fixture override.
        ns = {"__name__": "fixture"}
        exec(compile(SOURCE, "install.sh", "exec"), ns)
        for system, arch, expected in (
            ("Linux", "x86_64", "x86_64-unknown-linux-gnu"),
            ("Linux", "aarch64", "aarch64-unknown-linux-gnu"),
            ("Darwin", "x86_64", "x86_64-apple-darwin"),
            ("Darwin", "arm64", "aarch64-apple-darwin"),
        ):
            with patch("platform.system", return_value=system), patch("platform.machine", return_value=arch), patch("os.confstr", return_value="glibc 2.39"):
                self.assertEqual(ns["target"](), expected)
        with patch("platform.system", return_value="Linux"), patch("os.confstr", return_value=None):
            with self.assertRaisesRegex(ValueError, "glibc"):
                ns["target"]()
        with patch("platform.system", return_value="Windows"):
            with self.assertRaisesRegex(ValueError, "Linux and macOS"):
                ns["target"]()
        with patch("platform.system", return_value="Darwin"), patch("platform.machine", return_value="riscv64"):
            with self.assertRaisesRegex(ValueError, "CPU"):
                ns["target"]()


if __name__ == "__main__":
    unittest.main()

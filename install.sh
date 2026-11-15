#!/bin/sh
# Install a published Unix pxl binary. Python 3.9+ is required; no Rust build.
set -eu
exec python3 - "$@" <<'PY'
import argparse
import gzip
import hashlib
import io
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import tarfile
import urllib.parse
import urllib.request

REPO = "scbrown/pixelsrc"
MAX_ARCHIVE = 128 * 1024 * 1024
MAX_BINARY = 256 * 1024 * 1024
VERSION = re.compile(r"[0-9]+\.[0-9]+\.[0-9]+")


class HTTPSOnly(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        check_url(newurl)
        return super().redirect_request(req, fp, code, msg, headers, newurl)


def check_url(url):
    parsed = urllib.parse.urlsplit(url)
    if (parsed.scheme != "https" or parsed.username or parsed.password
            or parsed.port not in (None, 443)
            or parsed.hostname not in {
                "api.github.com", "github.com", "release-assets.githubusercontent.com",
                "objects.githubusercontent.com",
            }):
        raise ValueError("refusing download outside the GitHub HTTPS release hosts")


def fetch(url, limit):
    check_url(url)
    req = urllib.request.Request(url, headers={"User-Agent": "pixelsrc-installer"})
    with urllib.request.build_opener(HTTPSOnly()).open(req, timeout=30) as response:
        check_url(response.geturl())
        body = response.read(limit + 1)
    if len(body) > limit:
        raise ValueError("release response exceeds the size limit")
    return body


def target():
    system = platform.system()
    machine = platform.machine().lower()
    arch = {"x86_64": "x86_64", "amd64": "x86_64",
            "aarch64": "aarch64", "arm64": "aarch64"}.get(machine)
    if system == "Linux":
        try:
            glibc = os.confstr("CS_GNU_LIBC_VERSION")
        except (ValueError, OSError):
            glibc = None
        if not glibc:
            raise ValueError("Linux release binaries require glibc; use Cargo on musl")
        suffix = "unknown-linux-gnu"
    elif system == "Darwin":
        suffix = "apple-darwin"
    else:
        raise ValueError("this installer supports Linux and macOS only")
    if arch is None:
        raise ValueError("no published binary for this CPU architecture")
    return f"{arch}-{suffix}"


def release(version, triple):
    endpoint = (f"tags/v{version}" if version else "latest")
    info = json.loads(fetch(f"https://api.github.com/repos/{REPO}/releases/{endpoint}",
                            2 * 1024 * 1024))
    tag = info.get("tag_name", "")
    resolved = tag.removeprefix("v")
    if (not VERSION.fullmatch(resolved) or tag != f"v{resolved}"
            or info.get("draft") or info.get("prerelease")
            or (version and resolved != version)):
        raise ValueError("expected a stable vMAJOR.MINOR.PATCH release")
    archive = f"pxl-{tag}-{triple}.tar.gz"
    base = f"https://github.com/{REPO}/releases/download/{tag}/"
    for name in (archive, "SHA256SUMS.txt"):
        matches = [a for a in info.get("assets", []) if a.get("name") == name]
        if len(matches) != 1 or matches[0].get("browser_download_url") != base + name:
            raise ValueError(f"missing or unexpected release asset: {name}")
    return resolved, archive, base


def checked_binary(archive, sums, name):
    matches = []
    for line in sums.decode("utf-8").splitlines():
        fields = line.split()
        if len(fields) == 2 and fields[1].lstrip("*") == name:
            matches.append(fields[0])
    if len(matches) != 1 or not re.fullmatch(r"[0-9a-fA-F]{64}", matches[0]):
        raise ValueError("missing, duplicate or invalid archive checksum")
    if hashlib.sha256(archive).hexdigest() != matches[0].lower():
        raise ValueError("archive checksum mismatch")
    # Bound decompression before tarfile parses even extended/PAX headers.
    max_tar = MAX_BINARY + 1024 * 1024
    with gzip.GzipFile(fileobj=io.BytesIO(archive)) as compressed:
        uncompressed = compressed.read(max_tar + 1)
    if len(uncompressed) > max_tar:
        raise ValueError("decompressed archive exceeds the size limit")
    with tarfile.open(fileobj=io.BytesIO(uncompressed), mode="r:") as package:
        first = package.next()
        if (first is None or first.name != "pxl" or not first.isfile()
                or first.size <= 0 or first.size > MAX_BINARY
                or getattr(first, "sparse", None)):
            raise ValueError("expected one regular pxl file in the archive")
        stream = package.extractfile(first)
        binary = stream.read(MAX_BINARY + 1)
        if len(binary) != first.size or package.next() is not None:
            raise ValueError("unexpected archive content")
    return binary


def main(argv=None):
    parser = argparse.ArgumentParser(description="Install a checksum-verified stable pxl release")
    parser.add_argument("--version", help="stable version, for example 0.3.1")
    parser.add_argument("--prefix", type=Path, help="new private directory; no global command link")
    parser.add_argument("--dry-run", action="store_true", help="resolve metadata only, write nothing")
    args = parser.parse_args(argv)
    if args.version and not VERSION.fullmatch(args.version):
        raise ValueError("--version must be MAJOR.MINOR.PATCH")
    version, name, base = release(args.version, target())
    prefix = (args.prefix.expanduser().absolute() if args.prefix else
              Path.home() / ".local/share/pixelsrc" / f"v{version}")
    link = None if args.prefix else Path.home() / ".local/bin/pxl"
    if os.path.lexists(prefix):
        raise ValueError("prefix already exists; choose a new --prefix")
    if link and (os.path.lexists(link) or shutil.which("pxl")):
        raise ValueError("pxl already exists; use its package manager or an isolated --prefix")
    if args.dry_run:
        print(f"Would install {name} into {prefix}; metadata only, payload not verified")
        return
    archive = fetch(base + name, MAX_ARCHIVE)
    binary = checked_binary(archive, fetch(base + "SHA256SUMS.txt", 1024 * 1024), name)
    prefix.parent.mkdir(parents=True, exist_ok=True)
    prefix.mkdir()  # exclusive: never take ownership of an existing installation
    try:
        executable = prefix / "pxl"
        executable.write_bytes(binary)
        executable.chmod(0o755)
        child_env = {k: os.environ[k] for k in ("PATH", "HOME", "TMPDIR", "LANG")
                     if k in os.environ}
        result = subprocess.run([str(executable), "--version"], env=child_env,
                                capture_output=True, text=True, timeout=10, check=True)
        if result.stdout.strip() != f"pxl {version}":
            raise ValueError("installed binary reports an unexpected version")
        if link:
            link.parent.mkdir(parents=True, exist_ok=True)
            link.symlink_to(executable)
    except BaseException:
        shutil.rmtree(prefix)
        raise
    print(f"Installed pxl {version}: {executable}")
    if link:
        print(f"Command: {link}; add {link.parent} to PATH if needed")


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        # Network errors may contain URLs. The installer never accepts credentials.
        print(f"pxl install failed: {error}", file=sys.stderr)
        sys.exit(1)
PY

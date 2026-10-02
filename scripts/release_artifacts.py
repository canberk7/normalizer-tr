"""Inspect the exact release set; registry authentication is not handled here."""

import argparse
import email.parser
import itertools
import json
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import tarfile
import tomllib
import zipfile

from verification import inspect_wheel, sha, version, write

PYTHONS = ("3.11", "3.12", "3.13", "3.14")
PLATFORMS = ("windows", "linux", "macos-x64", "macos-arm64")


def check_members(names):
    for name in names:
        path = PurePosixPath(name)
        if (
            path.is_absolute()
            or "\\" in name
            or re.match(r"^[A-Za-z]:", name)
            or ".." in path.parts
            or set(path.parts)
            & {
                ".git",
                "target",
                "__pycache__",
                "integrations",
                ".venv",
                ".pytest_cache",
            }
            or path.suffix.lower() in {".pt", ".wav", ".exe", ".pem", ".key"}
            or path.name.startswith(".env")
            or path.name in {".pypirc", "credentials.toml"}
        ):
            raise RuntimeError(f"excluded or unsafe release entry: {name}")


def check_metadata(data):
    metadata = email.parser.Parser().parsestr(data.decode("utf-8"))
    if metadata["Name"] != "normalizer-tr" or metadata["Version"] != version():
        raise RuntimeError("distribution name/version differs from the core")
    requirement = metadata["Requires-Python"]
    if requirement is None or {part.strip() for part in requirement.split(",")} != {
        ">=3.11",
        "<3.15",
    }:
        raise RuntimeError("Python requirement differs from the tested matrix")
    if metadata.get_all("Requires-Dist"):
        raise RuntimeError("standalone binding acquired a runtime dependency")
    if metadata["License-Expression"] != "Apache-2.0":
        raise RuntimeError("owned license metadata differs")


def wheel_platform(tag):
    if tag == "win_amd64":
        return "windows"
    if "manylinux_2_28_x86_64" in tag.split("."):
        return "linux"
    if re.fullmatch(r"macosx_\d+_\d+_x86_64", tag):
        return "macos-x64"
    if re.fullmatch(r"macosx_\d+_\d+_arm64", tag):
        return "macos-arm64"
    raise RuntimeError(f"unapproved wheel platform tag: {tag}")


def inspect_python(path):
    if path.suffix == ".whl":
        inspect_wheel(path)
        match = re.fullmatch(
            r"normalizer_tr-(\d+\.\d+\.\d+)-(cp3\d{2})-\2-(.+)\.whl", path.name
        )
        if match is None or match[1] != version():
            raise RuntimeError(f"unexpected wheel filename: {path.name}")
        python = "3." + match[2][3:]
        if python not in PYTHONS:
            raise RuntimeError("wheel targets an unapproved interpreter")
        with zipfile.ZipFile(path) as archive:
            names = archive.namelist()
            check_members(names)
            check_metadata(
                archive.read(next(n for n in names if n.endswith("/METADATA")))
            )
            native = [n for n in names if n.endswith((".pyd", ".so"))]
            if len(native) != 1 or not native[0].startswith("normalizer_tr/_native."):
                raise RuntimeError("expected exactly one native normalizer extension")
        return {"kind": "wheel", "python": python, "platform": wheel_platform(match[3])}
    if path.name == f"normalizer_tr-{version()}.tar.gz":
        with tarfile.open(path) as archive:
            names = archive.getnames()
            check_members(names)
            infos = [n for n in names if n.endswith("/PKG-INFO")]
            if len(infos) != 1:
                raise RuntimeError("expected one source-distribution metadata file")
            check_metadata(archive.extractfile(infos[0]).read())
            if (
                sum(n.endswith("/Cargo.toml") for n in names) < 2
                or not any(n.endswith("/normalizer_tr/__init__.py") for n in names)
                or not any("LICENSE-UNICODE" in n for n in names)
            ):
                raise RuntimeError("sdist lacks the core, binding or required rights")
        return {"kind": "sdist"}
    raise RuntimeError(f"unexpected Python release artifact: {path.name}")


def artifact_record(path):
    return {
        "name": path.name,
        "sha256": sha(path),
        "bytes": path.stat().st_size,
        **inspect_python(path),
    }


def inspect(directory, python, platform, sdist):
    paths = sorted(directory.iterdir())
    if len(paths) != 1:
        raise RuntimeError("each build job must produce exactly one distribution")
    record = artifact_record(paths[0])
    if sdist:
        if record["kind"] != "sdist":
            raise RuntimeError("expected an sdist")
    elif (
        record["kind"] != "wheel"
        or record["python"] != python
        or record["platform"] != platform
    ):
        raise RuntimeError("built wheel differs from its test job")
    print(json.dumps(record, indent=2))


def manifest(directory, core_directory, output):
    artifacts = [artifact_record(path) for path in sorted(directory.iterdir())]
    actual = [
        (record["python"], record["platform"])
        for record in artifacts
        if record["kind"] == "wheel"
    ]
    expected = list(itertools.product(PYTHONS, PLATFORMS))
    if sorted(actual) != sorted(expected):
        raise RuntimeError("release wheel matrix is missing, duplicated or unexpected")
    if sum(record["kind"] == "sdist" for record in artifacts) != 1:
        raise RuntimeError("release requires exactly one tested sdist")
    head = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
    core = list(core_directory.glob(f"normalizer-tr-{version()}.crate"))
    if len(core) != 1:
        raise RuntimeError("expected one core archive")
    with tarfile.open(core[0]) as archive:
        names = archive.getnames()
        check_members(names)
        vcs = json.load(
            archive.extractfile(
                next(n for n in names if n.endswith(".cargo_vcs_info.json"))
            )
        )
        metadata = tomllib.loads(
            archive.extractfile(f"normalizer-tr-{version()}/Cargo.toml")
            .read()
            .decode("utf-8")
        )
        if vcs["git"]["sha1"] != head or metadata["package"]["publish"] != [
            "crates-io"
        ]:
            raise RuntimeError("core archive provenance/publication target differs")
    write(
        output,
        {
            "schema_version": 1,
            "version": version(),
            "source_head": head,
            "workflow_run_id": os.environ.get("GITHUB_RUN_ID"),
            "python_artifacts": artifacts,
            "core": {
                "name": core[0].name,
                "sha256": sha(core[0]),
                "bytes": core[0].stat().st_size,
                "vcs": vcs,
            },
            "matrix": {"python": PYTHONS, "platforms": PLATFORMS},
            "rust_toolchain": "1.99.0",
            "linux_baseline": "glibc 2.28 / manylinux_2_28",
        },
    )


def verify(directory, tag):
    record = json.loads(
        (directory / "release-manifest.json").read_text(encoding="utf-8")
    )
    head = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
    if (
        tag != "v" + version()
        or record["version"] != version()
        or record["source_head"] != head
    ):
        raise RuntimeError("release tag, version or source revision differs")
    paths = sorted((directory / "dist").iterdir())
    actual = [artifact_record(path) for path in paths]
    if actual != record["python_artifacts"]:
        raise RuntimeError("publish artifacts differ from the verified build set")
    core = directory / "target" / "release-core" / record["core"]["name"]
    if sha(core) != record["core"]["sha256"]:
        raise RuntimeError("core archive differs from the verified build")
    print(f"Verified {len(actual)} Python artifacts and core at {head}")


def main():
    parser = argparse.ArgumentParser()
    commands = parser.add_subparsers(dest="action", required=True)
    check = commands.add_parser("inspect")
    check.add_argument("directory", type=Path)
    check.add_argument("--python", choices=PYTHONS)
    check.add_argument("--platform", choices=PLATFORMS)
    check.add_argument("--sdist", action="store_true")
    collect = commands.add_parser("manifest")
    collect.add_argument("directory", type=Path)
    collect.add_argument("--core", type=Path, required=True)
    collect.add_argument("--output", type=Path, required=True)
    publish = commands.add_parser("verify")
    publish.add_argument("directory", type=Path)
    publish.add_argument("--tag", required=True)
    args = parser.parse_args()
    if args.action == "inspect":
        inspect(args.directory, args.python, args.platform, args.sdist)
    elif args.action == "manifest":
        manifest(args.directory, args.core, args.output)
    else:
        verify(args.directory, args.tag)


if __name__ == "__main__":
    main()

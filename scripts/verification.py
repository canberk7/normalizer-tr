"""Small stdlib-only helpers for one serial verification workflow; no engine actions."""

import argparse
import hashlib
import json
import os
import platform
import shutil
import subprocess
import tarfile
import tomllib
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def record(path):
    return {"path": str(path), "sha256": sha(path), "bytes": path.stat().st_size}


def write(path, value):
    with path.open("x", encoding="utf-8") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)


def version():
    return tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))["package"][
        "version"
    ]


def command(*args):
    return subprocess.check_output(args, cwd=ROOT, text=True).strip()


def prepare(output):
    release = version()
    archive = ROOT / "target" / "package" / f"normalizer-tr-{release}.crate"
    packages = output / "packages"
    packages.mkdir(exist_ok=True)
    archived = packages / archive.name
    shutil.copyfile(archive, archived)
    core = output / "core"
    core.mkdir()
    with tarfile.open(archived) as tar:
        names = tar.getnames()
        if any(
            any(
                part in name.split("/")
                for part in ("bindings", "integrations", "target", "__pycache__")
            )
            or name.endswith((".pt", ".wav", ".whl"))
            for name in names
        ):
            raise RuntimeError("core archive contains excluded content")
        tar.extractall(core, filter="data")
    package = core / f"normalizer-tr-{release}"
    vcs = json.loads((package / ".cargo_vcs_info.json").read_text(encoding="utf-8"))
    if vcs["git"]["sha1"] != command("git", "rev-parse", "HEAD"):
        raise RuntimeError("core archive VCS does not match clean source revision")
    if (
        tomllib.loads((package / "Cargo.toml").read_text(encoding="utf-8"))["package"][
            "publish"
        ]
        is not False
    ):
        raise RuntimeError("packaged publication guard is missing")
    consumer = output / "rust-consumer"
    (consumer / "src").mkdir(parents=True)
    (consumer / "Cargo.toml").write_text(
        "[workspace]\n"
        '[package]\nname="packaged-consumer"\nversion="0.0.0"\nedition="2024"\npublish=false\n'
        '[features]\ndefault=[]\nserde=["normalizer-tr/serde"]\n'
        f"[dependencies]\nnormalizer-tr={{path='{package.as_posix()}'}}\n",
        encoding="utf-8",
    )
    shutil.copyfile(ROOT / "scripts" / "core_consumer.rs", consumer / "src" / "lib.rs")
    write(
        output / "prepared.json",
        {
            "package_manifest": str(package / "Cargo.toml"),
            "consumer_manifest": str(consumer / "Cargo.toml"),
        },
    )


def wheel(output):
    candidates = list((output / "packages").glob("normalizer_tr_python-*.whl"))
    if len(candidates) != 1:
        raise RuntimeError("expected one authoritative wheel")
    path = candidates[0]
    with zipfile.ZipFile(path) as archive:
        names = archive.namelist()
        if any(
            name.startswith("integrations/") or name.endswith((".pt", ".wav"))
            for name in names
        ):
            raise RuntimeError("wheel includes engine/model assets")
        if not any("LICENSE-UNICODE" in name for name in names) or not any(
            "THIRD_PARTY_NOTICES" in name for name in names
        ):
            raise RuntimeError("compiled dependency/owned notices are absent")
        metadata = archive.read(
            next(name for name in names if name.endswith("/METADATA"))
        ).decode()
        if (
            "Requires-Dist: torch" in metadata
            or "Requires-Dist: huggingface" in metadata
        ):
            raise RuntimeError("wheel depends on engine packages")
        identity = archive.read("normalizer_tr/__init__.py").decode()
        if "LEGACY_RULESET_ID" in identity or "ruleset_id" in identity:
            raise RuntimeError("removed selector remains in the wheel facade")
    return path


def consume(output):
    import importlib.util
    import socket
    import normalizer_tr

    if (
        importlib.util.find_spec("torch") is not None
        or importlib.util.find_spec("huggingface_hub") is not None
    ):
        raise RuntimeError("fresh binding consumer must not require engine packages")
    if "site-packages" not in normalizer_tr.__file__:
        raise RuntimeError("consumer imported source tree instead of installed wheel")
    socket.socket = lambda *args, **kwargs: (_ for _ in ()).throw(
        RuntimeError("network disabled for consumer")
    )
    normalizer = normalizer_tr.Normalizer()
    if normalizer.normalize("25 TL").normalized_text != "yirmi beş Türk lirası":
        raise RuntimeError("installed-wheel IO differs")
    if hasattr(normalizer_tr, "LEGACY_RULESET_ID"):
        raise RuntimeError("removed public selector is still exported")
    write(
        output / "reports" / "python-consumer.json",
        {
            "normalizer_id": normalizer.normalizer_id,
            "native_sha256": sha(Path(normalizer_tr._native.__file__)),
            "python": platform.python_version(),
            "import_path": normalizer_tr.__file__,
            "network_used": False,
            "engine_required": False,
        },
    )


def finalize(output):
    stages = json.loads((output / "stages.json").read_text(encoding="utf-8-sig"))
    if any(stage["status"] != "passed" for stage in stages):
        raise RuntimeError("failed stage cannot become a successful verification")
    head = command("git", "rev-parse", "HEAD")
    if command("git", "status", "--porcelain"):
        raise RuntimeError("final source revision is not clean")
    native = []
    for path in sorted((output / "reports").glob("rust-*.json")):
        value = json.loads(path.read_text(encoding="utf-8"))
        if any(
            cohort["count"] < 10000 or cohort["p95_ns"] >= 1000000
            for cohort in value["cohorts"].values()
        ):
            raise RuntimeError("warm short/medium Rust p95 regression")
        native.append(
            {
                "file": record(path),
                "cohorts": value["cohorts"],
                "constructor_first_ns": value["constructor_first_process_init_ns"],
                "cached_constructor": value["constructors"],
                "per_class_policy": value["per_class_policy"],
                "large": value["large"],
                "limit_diagnostics": value["limit_diagnostics"],
            }
        )
    if len(native) != 3:
        raise RuntimeError("expected three measured native repetitions")
    package = output / "core" / f"normalizer-tr-{version()}"
    vcs = json.loads((package / ".cargo_vcs_info.json").read_text(encoding="utf-8"))
    if vcs["git"]["sha1"] != head:
        raise RuntimeError("source and packaged VCS differ")
    installed = json.loads(
        (output / "reports" / "python-consumer.json").read_text(encoding="utf-8")
    )
    authoritative = wheel(output)
    with zipfile.ZipFile(authoritative) as zipfile_:
        binary = zipfile_.read(
            next(name for name in zipfile_.namelist() if name.endswith(".pyd"))
        )
        if hashlib.sha256(binary).hexdigest() != installed["native_sha256"]:
            raise RuntimeError(
                "consumer native binary differs from authoritative wheel"
            )
    identity = installed["normalizer_id"]
    if any(
        json.loads(Path(row["file"]["path"]).read_text(encoding="utf-8"))[
            "normalizer_id"
        ]
        != identity
        for row in native
    ):
        raise RuntimeError("measurement/consumer identity mismatch")
    manifest = {
        "schema_version": 1,
        "status": "passed",
        "source_head": head,
        "normalizer_id": identity,
        "compiler": command("rustc", "-vV"),
        "cargo": command("cargo", "--version"),
        "build": {
            "profile": "release/bench",
            "features": ["serde"],
            "rustflags": os.environ.get("RUSTFLAGS", ""),
        },
        "host": json.loads((output / "host.json").read_text(encoding="utf-8-sig")),
        "stages": stages,
        "packages": [record(path) for path in sorted((output / "packages").iterdir())],
        "core_archive_vcs": vcs,
        "native_repetitions": native,
        "python_report": record(output / "reports" / "python.json"),
        "corpora": [
            record(ROOT / "benches" / name)
            for name in ("corpus.json", "intent-corpus.json")
        ],
        "consumer": installed,
        "artifact_files": [
            record(path) for path in sorted((output / "reports").iterdir())
        ],
        "limits": "warm per-cohort p95 only; cold/large/Python/concurrency separate; no native heap instrumentation",
        "scope": {
            "model_actions": False,
            "publication": False,
            "global_settings_changed": False,
        },
    }
    write(output / "manifest.json", manifest)
    lines = [
        f"PASS normalizer {identity}",
        f"Source {head}",
        "Warm Rust p95 regression (<1ms per cohort):",
    ]
    for index, row in enumerate(native, 1):
        lines.append(
            f"  repetition{index}: short {row['cohorts']['short']['p95_ns'] / 1000:g}us; medium {row['cohorts']['medium']['p95_ns'] / 1000:g}us"
        )
    lines += [
        "No all-input/cold/Python/audio1ms claim.",
        "Packages, host, tests, class/policy quantiles and exact checksums: manifest.json",
    ]
    with (output / "summary.txt").open("x", encoding="utf-8") as stream:
        stream.write("\n".join(lines) + "\n")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=("prepare", "wheel", "consume", "finalize"))
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    {"prepare": prepare, "wheel": wheel, "consume": consume, "finalize": finalize}[
        args.action
    ](args.output)


if __name__ == "__main__":
    main()

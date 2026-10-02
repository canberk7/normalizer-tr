"""Verification machinery checks; no models or external source access."""

import json
import tarfile
import tempfile
import tomllib
import unittest
from pathlib import Path
from unittest.mock import patch
from verification import write, wheel, prepare


class VerificationTests(unittest.TestCase):
    def test_reports_refuse_overwrite(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "report.json"
            write(path, {"status": "failed"})
            with self.assertRaises(FileExistsError):
                write(path, {"status": "passed"})
            self.assertEqual(json.loads(path.read_text()), {"status": "failed"})

    def test_missing_or_ambiguous_wheel_is_not_success(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "packages").mkdir()
            with self.assertRaises(RuntimeError):
                wheel(root)

    def test_generated_consumer_owns_its_workspace(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "Cargo.toml").write_text(
                '[package]\nversion="0.3.0"\n', encoding="utf-8"
            )
            (root / "scripts").mkdir()
            (root / "scripts" / "core_consumer.rs").write_text("", encoding="utf-8")
            fixture = root / "fixture"
            fixture.mkdir()
            (fixture / "Cargo.toml").write_text(
                '[package]\npublish=["crates-io"]\n', encoding="utf-8"
            )
            (fixture / ".cargo_vcs_info.json").write_text(
                json.dumps({"git": {"sha1": "a" * 40}}), encoding="utf-8"
            )
            archives = root / "target" / "package"
            archives.mkdir(parents=True)
            with tarfile.open(archives / "normalizer-tr-0.3.0.crate", "w:gz") as tar:
                tar.add(fixture, arcname="normalizer-tr-0.3.0")
            output = root / "target" / "verification"
            output.mkdir()
            with (
                patch("verification.ROOT", root),
                patch("verification.command", return_value="a" * 40),
            ):
                prepare(output)
            manifest = tomllib.loads(
                (output / "rust-consumer" / "Cargo.toml").read_text(encoding="utf-8")
            )
            self.assertEqual(manifest["workspace"], {})


if __name__ == "__main__":
    unittest.main()

"""Release gate regressions; no registry, native execution or credentials."""

import unittest

from release_artifacts import check_members, check_metadata, wheel_platform

METADATA = (
    "Name: normalizer-tr\n"
    "Version: 0.4.0\n"
    "Requires-Python: <3.15, >=3.11\n"
    "License-Expression: Apache-2.0\n"
)


class ReleaseTests(unittest.TestCase):
    def test_metadata_accepts_normalized_requirement_order_and_spacing(self):
        check_metadata(METADATA.encode())

    def test_metadata_cannot_weaken_the_matrix_or_add_runtime_dependencies(self):
        for text in (
            METADATA.replace(">=3.11", ">=3.10"),
            METADATA.replace("normalizer-tr", "different-package"),
            METADATA.replace("0.4.0", "0.4.1"),
            METADATA + "Requires-Dist: torch\n",
        ):
            with self.subTest(text=text), self.assertRaises(RuntimeError):
                check_metadata(text.encode())

    def test_archive_excludes_secrets_models_and_cross_platform_traversal(self):
        check_members(["normalizer_tr/__init__.py", "licenses/COPYRIGHT-library.html"])
        for name in (
            "../outside",
            r"..\outside",
            r"C:\outside",
            "/outside",
            "normalizer_tr/.env.production",
            "credentials.toml",
            "integrations/example.py",
            "decoder.pt",
            ".git/config",
        ):
            with self.subTest(name=name), self.assertRaises(RuntimeError):
                check_members([name])

    def test_platform_matrix_does_not_claim_other_architectures(self):
        for tag, expected in (
            ("win_amd64", "windows"),
            ("manylinux_2_28_x86_64", "linux"),
            ("macosx_12_0_x86_64", "macos-x64"),
            ("macosx_12_0_arm64", "macos-arm64"),
        ):
            self.assertEqual(wheel_platform(tag), expected)
        for tag in ("win_arm64", "musllinux_1_2_x86_64", "manylinux_2_28_aarch64"):
            with self.subTest(tag=tag), self.assertRaises(RuntimeError):
                wheel_platform(tag)


if __name__ == "__main__":
    unittest.main()

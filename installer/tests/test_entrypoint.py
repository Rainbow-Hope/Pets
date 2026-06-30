from __future__ import annotations

import os
import subprocess
import sys
import unittest
from pathlib import Path


class EntrypointTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.installer_root = Path(__file__).resolve().parents[1]
        cls.environment = {
            **os.environ,
            "PYTHONPATH": str(cls.installer_root),
        }

    def test_self_test_exits_successfully_without_opening_gui(self) -> None:
        result = subprocess.run(
            [sys.executable, "-m", "pet_installer", "--self-test"],
            cwd=self.installer_root,
            env=self.environment,
            capture_output=True,
            text=True,
            timeout=15,
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("SELF_TEST_OK", result.stdout)

    def test_unknown_argument_returns_usage_error(self) -> None:
        result = subprocess.run(
            [sys.executable, "-m", "pet_installer", "--desconhecido"],
            cwd=self.installer_root,
            env=self.environment,
            capture_output=True,
            text=True,
            timeout=15,
        )

        self.assertEqual(result.returncode, 2)
        self.assertIn("usage:", result.stderr.lower())

    def test_pyinstaller_wrapper_runs_self_test(self) -> None:
        result = subprocess.run(
            [
                sys.executable,
                str(self.installer_root / "run_installer.py"),
                "--self-test",
            ],
            cwd=self.installer_root,
            env=self.environment,
            capture_output=True,
            text=True,
            timeout=15,
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("SELF_TEST_OK", result.stdout)


if __name__ == "__main__":
    unittest.main()

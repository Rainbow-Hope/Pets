from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from installer.pet_installer.installation import (
    InstallationError,
    install_copy,
    install_new,
    slugify_display_name,
    update_existing,
)
from installer.pet_installer.package import validate_pet_zip
from installer.tests.helpers import write_pet_zip


class InstallationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        self.pets_root = self.root / "pets"
        self.zip_path = write_pet_zip(self.root / "rainbow-hope.zip")
        self.package = validate_pet_zip(self.zip_path)

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def test_installs_new_pet(self) -> None:
        installed = install_new(self.package, self.pets_root)

        self.assertEqual(installed, self.pets_root / "rainbow-hope")
        self.assertTrue((installed / "pet.json").is_file())
        self.assertTrue((installed / "spritesheet.webp").is_file())
        self.assertEqual(self._temporary_entries(), [])

    def test_rejects_existing_destination(self) -> None:
        destination = self.pets_root / "rainbow-hope"
        destination.mkdir(parents=True)
        (destination / "marker.txt").write_text("original", encoding="utf-8")

        with self.assertRaisesRegex(InstallationError, "ja existe"):
            install_new(self.package, self.pets_root)

        self.assertEqual(
            (destination / "marker.txt").read_text(encoding="utf-8"),
            "original",
        )

    def test_updates_existing_pet(self) -> None:
        destination = self.pets_root / "rainbow-hope"
        destination.mkdir(parents=True)
        (destination / "pet.json").write_text("old", encoding="utf-8")
        (destination / "old.txt").write_text("old", encoding="utf-8")

        updated = update_existing(self.package, self.pets_root)

        self.assertEqual(updated, destination)
        self.assertFalse((destination / "old.txt").exists())
        self.assertEqual(
            json.loads((destination / "pet.json").read_text(encoding="utf-8"))[
                "id"
            ],
            "rainbow-hope",
        )
        self.assertEqual(self._temporary_entries(), [])

    def test_failed_update_restores_original_pet(self) -> None:
        destination = self.pets_root / "rainbow-hope"
        destination.mkdir(parents=True)
        (destination / "marker.txt").write_text("original", encoding="utf-8")
        real_rename = Path.rename
        calls = 0

        def fail_second_rename(source: Path, target: Path) -> Path:
            nonlocal calls
            calls += 1
            if calls == 2:
                raise OSError("falha simulada")
            return real_rename(source, target)

        with mock.patch(
            "installer.pet_installer.installation._rename",
            side_effect=fail_second_rename,
        ):
            with self.assertRaisesRegex(InstallationError, "restaurado"):
                update_existing(self.package, self.pets_root)

        self.assertEqual(
            (destination / "marker.txt").read_text(encoding="utf-8"),
            "original",
        )
        self.assertEqual(self._temporary_entries(), [])

    def test_installs_copy_with_rewritten_id_and_display_name(self) -> None:
        installed = install_copy(
            self.package,
            self.pets_root,
            "Rainbow Hope Azul",
        )

        self.assertEqual(installed, self.pets_root / "rainbow-hope-azul")
        manifest = json.loads(
            (installed / "pet.json").read_text(encoding="utf-8")
        )
        self.assertEqual(manifest["id"], "rainbow-hope-azul")
        self.assertEqual(manifest["displayName"], "Rainbow Hope Azul")
        self.assertEqual(
            manifest["spritesheetPath"],
            "spritesheet.webp",
        )

    def test_copy_rejects_invalid_name(self) -> None:
        for name in ("", "---", "   "):
            with self.subTest(name=name):
                with self.assertRaisesRegex(InstallationError, "nome"):
                    install_copy(self.package, self.pets_root, name)

    def test_copy_rejects_conflicting_id(self) -> None:
        destination = self.pets_root / "rainbow-hope-azul"
        destination.mkdir(parents=True)

        with self.assertRaisesRegex(InstallationError, "ja existe"):
            install_copy(self.package, self.pets_root, "Rainbow Hope Azul")

    def test_slugifies_accents_and_spacing(self) -> None:
        self.assertEqual(
            slugify_display_name("  Fusão   Azul  "),
            "fusao-azul",
        )

    def _temporary_entries(self) -> list[str]:
        if not self.pets_root.exists():
            return []
        return sorted(
            path.name
            for path in self.pets_root.iterdir()
            if path.name.startswith((".install-", ".backup-"))
        )


if __name__ == "__main__":
    unittest.main()

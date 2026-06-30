from __future__ import annotations

import tempfile
import unittest
import warnings
import zipfile
from pathlib import Path
from unittest import mock

from installer.tests.helpers import (
    pet_manifest,
    write_pet_zip,
    write_symlink_entry,
    write_zip,
)
from installer.pet_installer.package import (
    PackageError,
    extract_package,
    validate_pet_zip,
)


class ValidatePetZipTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def test_accepts_repository_style_package(self) -> None:
        zip_path = write_pet_zip(self.root / "rainbow-hope.zip")

        package = validate_pet_zip(zip_path)

        self.assertEqual(package.pet_id, "rainbow-hope")
        self.assertEqual(package.display_name, "Rainbow Hope")
        self.assertEqual(package.root_name, "rainbow-hope")
        self.assertEqual(package.spritesheet_path, "spritesheet.webp")
        self.assertEqual(set(package.files), {"pet.json", "spritesheet.webp"})

    def test_extracts_only_validated_files(self) -> None:
        zip_path = write_pet_zip(
            self.root / "rainbow-hope.zip",
            files={"rainbow-hope/notes/info.txt": b"ok"},
        )
        package = validate_pet_zip(zip_path)
        destination = self.root / "staging"

        extract_package(package, destination)

        self.assertEqual((destination / "notes" / "info.txt").read_bytes(), b"ok")
        self.assertTrue((destination / "pet.json").is_file())
        self.assertTrue((destination / "spritesheet.webp").is_file())

    def test_rejects_missing_pet_json(self) -> None:
        zip_path = write_zip(
            self.root / "missing-json.zip",
            {"rainbow-hope/spritesheet.webp": b"webp"},
        )
        with self.assertRaisesRegex(PackageError, "pet.json"):
            validate_pet_zip(zip_path)

    def test_rejects_invalid_pet_json(self) -> None:
        zip_path = write_zip(
            self.root / "invalid-json.zip",
            {
                "rainbow-hope/pet.json": b"{",
                "rainbow-hope/spritesheet.webp": b"webp",
            },
        )
        with self.assertRaisesRegex(PackageError, "JSON"):
            validate_pet_zip(zip_path)

    def test_rejects_missing_spritesheet(self) -> None:
        zip_path = write_zip(
            self.root / "missing-sheet.zip",
            {"rainbow-hope/pet.json": pet_manifest()},
        )
        with self.assertRaisesRegex(PackageError, "spritesheet"):
            validate_pet_zip(zip_path)

    def test_rejects_invalid_pet_id(self) -> None:
        zip_path = write_zip(
            self.root / "bad-id.zip",
            {
                "bad-id/pet.json": pet_manifest("Bad ID"),
                "bad-id/spritesheet.webp": b"webp",
            },
        )
        with self.assertRaisesRegex(PackageError, "ID"):
            validate_pet_zip(zip_path)

    def test_rejects_parent_traversal(self) -> None:
        zip_path = write_pet_zip(
            self.root / "traversal.zip",
            files={"rainbow-hope/../escape.txt": b"bad"},
        )
        with self.assertRaisesRegex(PackageError, "inseguro"):
            validate_pet_zip(zip_path)

    def test_rejects_absolute_path(self) -> None:
        zip_path = write_pet_zip(
            self.root / "absolute.zip",
            files={"/escape.txt": b"bad"},
        )
        with self.assertRaisesRegex(PackageError, "inseguro"):
            validate_pet_zip(zip_path)

    def test_rejects_drive_qualified_path(self) -> None:
        zip_path = write_pet_zip(
            self.root / "drive.zip",
            files={"C:/escape.txt": b"bad"},
        )
        with self.assertRaisesRegex(PackageError, "inseguro"):
            validate_pet_zip(zip_path)

    def test_rejects_duplicate_entries(self) -> None:
        zip_path = self.root / "duplicate.zip"
        with warnings.catch_warnings():
            warnings.simplefilter("ignore", UserWarning)
            with zipfile.ZipFile(zip_path, "w") as archive:
                archive.writestr("rainbow-hope/pet.json", pet_manifest())
                archive.writestr("rainbow-hope/pet.json", pet_manifest())
                archive.writestr("rainbow-hope/spritesheet.webp", b"webp")
        with self.assertRaisesRegex(PackageError, "duplicad"):
            validate_pet_zip(zip_path)

    def test_rejects_multiple_top_level_roots(self) -> None:
        zip_path = write_pet_zip(
            self.root / "roots.zip",
            files={"other/file.txt": b"bad"},
        )
        with self.assertRaisesRegex(PackageError, "pasta principal"):
            validate_pet_zip(zip_path)

    def test_rejects_symbolic_links(self) -> None:
        zip_path = write_symlink_entry(self.root / "link.zip")
        with self.assertRaisesRegex(PackageError, "link"):
            validate_pet_zip(zip_path)

    def test_rejects_too_many_files(self) -> None:
        zip_path = write_pet_zip(self.root / "many.zip")
        with mock.patch("installer.pet_installer.package.MAX_FILES", 1):
            with self.assertRaisesRegex(PackageError, "arquivos"):
                validate_pet_zip(zip_path)

    def test_rejects_oversized_uncompressed_content(self) -> None:
        zip_path = write_pet_zip(self.root / "large.zip")
        with mock.patch(
            "installer.pet_installer.package.MAX_UNCOMPRESSED_BYTES",
            4,
        ):
            with self.assertRaisesRegex(PackageError, "tamanho"):
                validate_pet_zip(zip_path)


if __name__ == "__main__":
    unittest.main()

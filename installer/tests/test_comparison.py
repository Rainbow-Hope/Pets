from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from types import MappingProxyType

from installer.pet_installer.comparison import (
    compare_size_maps,
    directory_size_map,
    package_size_map,
)
from installer.pet_installer.package import PetPackage


class SizeComparisonTests(unittest.TestCase):
    def test_identical_maps_match_after_individual_file_check(self) -> None:
        result = compare_size_maps(
            {"pet.json": 80, "spritesheet.webp": 160},
            {"pet.json": 80, "spritesheet.webp": 160},
        )

        self.assertTrue(result.identical)
        self.assertEqual(result.incoming_total_bits, 240)
        self.assertEqual(result.installed_total_bits, 240)
        self.assertEqual(result.stage, "files")
        self.assertEqual(result.differing_paths, ())

    def test_different_totals_stop_at_total_stage(self) -> None:
        result = compare_size_maps(
            {"pet.json": 80, "spritesheet.webp": 160},
            {"pet.json": 80, "spritesheet.webp": 168},
        )

        self.assertFalse(result.identical)
        self.assertEqual(result.stage, "total")
        self.assertEqual(result.incoming_total_bits, 240)
        self.assertEqual(result.installed_total_bits, 248)

    def test_equal_totals_with_different_paths_are_distinct(self) -> None:
        result = compare_size_maps(
            {"pet.json": 80, "spritesheet.webp": 160},
            {"pet.json": 80, "other.webp": 160},
        )

        self.assertFalse(result.identical)
        self.assertEqual(result.stage, "paths")
        self.assertEqual(
            result.differing_paths,
            ("other.webp", "spritesheet.webp"),
        )

    def test_equal_totals_and_paths_compare_each_size(self) -> None:
        result = compare_size_maps(
            {"pet.json": 80, "spritesheet.webp": 160},
            {"pet.json": 100, "spritesheet.webp": 140},
        )

        self.assertFalse(result.identical)
        self.assertEqual(result.stage, "files")
        self.assertEqual(
            result.differing_paths,
            ("pet.json", "spritesheet.webp"),
        )

    def test_package_sizes_are_converted_from_bytes_to_bits(self) -> None:
        package = PetPackage(
            archive_path=Path("pet.zip"),
            root_name="pet",
            pet_id="pet",
            display_name="Pet",
            spritesheet_path="spritesheet.webp",
            files=MappingProxyType({"pet.json": 10, "spritesheet.webp": 20}),
        )

        self.assertEqual(
            package_size_map(package),
            {"pet.json": 80, "spritesheet.webp": 160},
        )

    def test_directory_map_uses_relative_posix_paths_and_bits(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root / "pet.json").write_bytes(b"12345")
            (root / "notes").mkdir()
            (root / "notes" / "info.txt").write_bytes(b"123")

            result = directory_size_map(root)

        self.assertEqual(
            result,
            {"notes/info.txt": 24, "pet.json": 40},
        )


if __name__ == "__main__":
    unittest.main()

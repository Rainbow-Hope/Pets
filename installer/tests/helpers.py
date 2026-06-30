from __future__ import annotations

import json
import stat
import zipfile
from pathlib import Path
from typing import Mapping


def pet_manifest(
    pet_id: str = "rainbow-hope",
    display_name: str = "Rainbow Hope",
    spritesheet_path: str = "spritesheet.webp",
) -> bytes:
    return (
        json.dumps(
            {
                "id": pet_id,
                "displayName": display_name,
                "description": "Pet de teste",
                "spritesheetPath": spritesheet_path,
            },
            ensure_ascii=False,
            indent=2,
        )
        + "\n"
    ).encode("utf-8")


def write_zip(path: Path, entries: Mapping[str, bytes]) -> Path:
    with zipfile.ZipFile(path, "w", compression=zipfile.ZIP_STORED) as archive:
        for name, content in entries.items():
            archive.writestr(name, content)
    return path


def write_pet_zip(
    path: Path,
    *,
    pet_id: str = "rainbow-hope",
    display_name: str = "Rainbow Hope",
    files: Mapping[str, bytes] | None = None,
) -> Path:
    entries = {
        f"{pet_id}/pet.json": pet_manifest(pet_id, display_name),
        f"{pet_id}/spritesheet.webp": b"RIFF-test-WEBP",
    }
    if files:
        entries.update(files)
    return write_zip(path, entries)


def write_symlink_entry(path: Path) -> Path:
    with zipfile.ZipFile(path, "w") as archive:
        archive.writestr(
            "rainbow-hope/pet.json",
            pet_manifest(),
        )
        archive.writestr("rainbow-hope/spritesheet.webp", b"RIFF-test-WEBP")
        link = zipfile.ZipInfo("rainbow-hope/link")
        link.create_system = 3
        link.external_attr = (stat.S_IFLNK | 0o777) << 16
        archive.writestr(link, b"spritesheet.webp")
    return path

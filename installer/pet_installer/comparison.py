from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Literal, Mapping

from .package import PetPackage


@dataclass(frozen=True)
class SizeComparison:
    identical: bool
    stage: Literal["total", "paths", "files"]
    incoming_total_bits: int
    installed_total_bits: int
    differing_paths: tuple[str, ...]


def package_size_map(package: PetPackage) -> dict[str, int]:
    return {
        path: byte_size * 8
        for path, byte_size in sorted(package.files.items())
    }


def directory_size_map(directory: Path) -> dict[str, int]:
    directory = Path(directory)
    if not directory.is_dir():
        return {}
    result: dict[str, int] = {}
    for path in sorted(directory.rglob("*")):
        if path.is_file() and not path.is_symlink():
            relative = path.relative_to(directory).as_posix()
            result[relative] = path.stat().st_size * 8
    return result


def compare_size_maps(
    incoming_bits: Mapping[str, int],
    installed_bits: Mapping[str, int],
) -> SizeComparison:
    incoming_total = sum(incoming_bits.values())
    installed_total = sum(installed_bits.values())
    if incoming_total != installed_total:
        return SizeComparison(
            identical=False,
            stage="total",
            incoming_total_bits=incoming_total,
            installed_total_bits=installed_total,
            differing_paths=(),
        )

    incoming_paths = set(incoming_bits)
    installed_paths = set(installed_bits)
    if incoming_paths != installed_paths:
        return SizeComparison(
            identical=False,
            stage="paths",
            incoming_total_bits=incoming_total,
            installed_total_bits=installed_total,
            differing_paths=tuple(sorted(incoming_paths ^ installed_paths)),
        )

    differing_paths = tuple(
        sorted(
            path
            for path in incoming_paths
            if incoming_bits[path] != installed_bits[path]
        )
    )
    return SizeComparison(
        identical=not differing_paths,
        stage="files",
        incoming_total_bits=incoming_total,
        installed_total_bits=installed_total,
        differing_paths=differing_paths,
    )

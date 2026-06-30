from __future__ import annotations

import json
import re
import shutil
import stat
import zipfile
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from types import MappingProxyType
from typing import Mapping

MAX_FILES = 100
MAX_UNCOMPRESSED_BYTES = 100 * 1024 * 1024
PET_ID_PATTERN = re.compile(r"^[a-z0-9](?:[a-z0-9-]{0,62}[a-z0-9])?$")
DRIVE_PATTERN = re.compile(r"^[A-Za-z]:")


class PackageError(ValueError):
    """Raised when a selected ZIP is not a safe, valid pet package."""


@dataclass(frozen=True)
class PetPackage:
    archive_path: Path
    root_name: str
    pet_id: str
    display_name: str
    spritesheet_path: str
    files: Mapping[str, int]


def _safe_parts(name: str) -> tuple[str, ...]:
    normalized = name.replace("\\", "/")
    if (
        not normalized
        or "\x00" in normalized
        or normalized.startswith("/")
        or DRIVE_PATTERN.match(normalized)
    ):
        raise PackageError(f"Caminho inseguro no ZIP: {name}")
    path = PurePosixPath(normalized)
    if path.is_absolute() or any(part in {"", ".", ".."} for part in path.parts):
        raise PackageError(f"Caminho inseguro no ZIP: {name}")
    return path.parts


def _is_link_or_special(info: zipfile.ZipInfo) -> bool:
    if info.create_system != 3:
        return False
    file_type = stat.S_IFMT(info.external_attr >> 16)
    return file_type not in {0, stat.S_IFREG, stat.S_IFDIR}


def _validated_members(
    archive: zipfile.ZipFile,
) -> tuple[str, dict[str, zipfile.ZipInfo]]:
    members: dict[str, zipfile.ZipInfo] = {}
    casefolded_names: set[str] = set()
    roots: set[str] = set()
    total_size = 0

    for info in archive.infolist():
        parts = _safe_parts(info.filename)
        if info.flag_bits & 0x1:
            raise PackageError("O ZIP contem arquivo criptografado.")
        if _is_link_or_special(info):
            raise PackageError(f"O ZIP contem link ou arquivo especial: {info.filename}")
        if info.is_dir():
            continue
        if len(parts) < 2:
            raise PackageError("O ZIP deve conter uma unica pasta principal.")

        normalized = "/".join(parts)
        folded = normalized.casefold()
        if folded in casefolded_names:
            raise PackageError(f"Entrada duplicada no ZIP: {normalized}")
        casefolded_names.add(folded)
        members[normalized] = info
        roots.add(parts[0])
        total_size += info.file_size

        if len(members) > MAX_FILES:
            raise PackageError(f"O ZIP excede o limite de {MAX_FILES} arquivos.")
        if total_size > MAX_UNCOMPRESSED_BYTES:
            raise PackageError("O ZIP excede o limite de tamanho descompactado.")

    if not members:
        raise PackageError("O ZIP esta vazio.")
    if len(roots) != 1:
        raise PackageError("O ZIP deve conter uma unica pasta principal.")
    return next(iter(roots)), members


def _read_manifest(
    archive: zipfile.ZipFile,
    root_name: str,
    members: Mapping[str, zipfile.ZipInfo],
) -> dict[str, object]:
    manifest_name = f"{root_name}/pet.json"
    info = members.get(manifest_name)
    if info is None:
        raise PackageError("O pacote nao contem pet.json na pasta principal.")
    try:
        raw = archive.read(info).decode("utf-8")
        manifest = json.loads(raw)
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise PackageError("O pet.json nao e um JSON UTF-8 valido.") from exc
    if not isinstance(manifest, dict):
        raise PackageError("O pet.json deve conter um objeto JSON.")
    return manifest


def _required_text(manifest: Mapping[str, object], field: str) -> str:
    value = manifest.get(field)
    if not isinstance(value, str) or not value.strip():
        raise PackageError(f"O pet.json nao possui {field} valido.")
    return value.strip()


def _safe_relative_path(value: str, field: str) -> str:
    parts = _safe_parts(value)
    if len(parts) == 0:
        raise PackageError(f"O campo {field} possui caminho invalido.")
    return "/".join(parts)


def _package_from_open_archive(
    archive: zipfile.ZipFile,
    archive_path: Path,
) -> tuple[PetPackage, dict[str, zipfile.ZipInfo]]:
    root_name, members = _validated_members(archive)
    manifest = _read_manifest(archive, root_name, members)
    pet_id = _required_text(manifest, "id")
    if not PET_ID_PATTERN.fullmatch(pet_id):
        raise PackageError("O ID do pet e invalido.")
    display_name = _required_text(manifest, "displayName")
    spritesheet_path = _safe_relative_path(
        _required_text(manifest, "spritesheetPath"),
        "spritesheetPath",
    )
    spritesheet_member = f"{root_name}/{spritesheet_path}"
    if spritesheet_member not in members:
        raise PackageError("O spritesheet indicado no pet.json nao existe no ZIP.")

    relative_files = {
        name.removeprefix(f"{root_name}/"): info.file_size
        for name, info in members.items()
    }
    return (
        PetPackage(
            archive_path=archive_path.resolve(),
            root_name=root_name,
            pet_id=pet_id,
            display_name=display_name,
            spritesheet_path=spritesheet_path,
            files=MappingProxyType(relative_files),
        ),
        members,
    )


def validate_pet_zip(path: str | Path) -> PetPackage:
    archive_path = Path(path)
    if not archive_path.is_file():
        raise PackageError(f"Arquivo ZIP nao encontrado: {archive_path}")
    try:
        with zipfile.ZipFile(archive_path, "r") as archive:
            package, _ = _package_from_open_archive(archive, archive_path)
            return package
    except zipfile.BadZipFile as exc:
        raise PackageError("O arquivo selecionado nao e um ZIP valido.") from exc


def extract_package(package: PetPackage, destination: Path) -> None:
    destination = Path(destination)
    if destination.exists() and any(destination.iterdir()):
        raise PackageError(f"A pasta temporaria nao esta vazia: {destination}")
    destination.mkdir(parents=True, exist_ok=True)

    try:
        with zipfile.ZipFile(package.archive_path, "r") as archive:
            current, members = _package_from_open_archive(
                archive,
                package.archive_path,
            )
            expected_identity = (
                package.root_name,
                package.pet_id,
                package.display_name,
                package.spritesheet_path,
                dict(package.files),
            )
            current_identity = (
                current.root_name,
                current.pet_id,
                current.display_name,
                current.spritesheet_path,
                dict(current.files),
            )
            if current_identity != expected_identity:
                raise PackageError(
                    "O pacote foi alterado depois da validacao. "
                    "Selecione o ZIP novamente."
                )
            for relative_name in package.files:
                member_name = f"{package.root_name}/{relative_name}"
                info = members.get(member_name)
                if info is None:
                    raise PackageError(
                        f"Arquivo validado desapareceu do ZIP: {relative_name}"
                    )
                output_path = destination.joinpath(*PurePosixPath(relative_name).parts)
                output_path.parent.mkdir(parents=True, exist_ok=True)
                with archive.open(info, "r") as source, output_path.open("wb") as target:
                    shutil.copyfileobj(source, target)
    except zipfile.BadZipFile as exc:
        raise PackageError("O arquivo ZIP foi alterado ou corrompido.") from exc

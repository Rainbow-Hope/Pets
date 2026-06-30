from __future__ import annotations

import json
import re
import shutil
import unicodedata
import uuid
from pathlib import Path

from .package import PET_ID_PATTERN, PackageError, PetPackage, extract_package


class InstallationError(RuntimeError):
    """Raised when a validated pet cannot be installed safely."""


def default_pets_root() -> Path:
    return Path.home() / ".codex" / "pets"


def slugify_display_name(display_name: str) -> str:
    normalized = unicodedata.normalize("NFKD", display_name)
    ascii_name = normalized.encode("ascii", "ignore").decode("ascii").lower()
    slug = re.sub(r"[^a-z0-9]+", "-", ascii_name).strip("-")
    if not slug or not PET_ID_PATTERN.fullmatch(slug):
        raise InstallationError("O novo nome nao gera um ID de pet valido.")
    return slug


def _unique_path(pets_root: Path, prefix: str, pet_id: str) -> Path:
    return pets_root / f".{prefix}-{pet_id}-{uuid.uuid4().hex}"


def _rename(source: Path, target: Path) -> Path:
    return source.rename(target)


def _remove_path(path: Path) -> None:
    if path.is_dir():
        shutil.rmtree(path)
    elif path.exists():
        path.unlink()


def _stage_package(package: PetPackage, pets_root: Path) -> Path:
    pets_root.mkdir(parents=True, exist_ok=True)
    staging = _unique_path(pets_root, "install", package.pet_id)
    try:
        extract_package(package, staging)
    except (OSError, PackageError) as exc:
        _remove_path(staging)
        raise InstallationError(f"Nao foi possivel preparar o pet: {exc}") from exc
    return staging


def install_new(package: PetPackage, pets_root: Path) -> Path:
    pets_root = Path(pets_root)
    destination = pets_root / package.pet_id
    if destination.exists():
        raise InstallationError(f"O pet {package.pet_id} ja existe.")

    staging = _stage_package(package, pets_root)
    try:
        _rename(staging, destination)
    except OSError as exc:
        _remove_path(staging)
        raise InstallationError(
            f"Nao foi possivel instalar o pet em {destination}: {exc}"
        ) from exc
    return destination


def update_existing(package: PetPackage, pets_root: Path) -> Path:
    pets_root = Path(pets_root)
    destination = pets_root / package.pet_id
    if not destination.is_dir():
        raise InstallationError(f"O pet {package.pet_id} nao esta instalado.")

    staging = _stage_package(package, pets_root)
    backup = _unique_path(pets_root, "backup", package.pet_id)
    try:
        _rename(destination, backup)
    except OSError as exc:
        _remove_path(staging)
        raise InstallationError(
            f"Nao foi possivel criar o backup de {destination}: {exc}"
        ) from exc

    try:
        _rename(staging, destination)
    except OSError as install_exc:
        _remove_path(staging)
        try:
            _rename(backup, destination)
        except OSError as restore_exc:
            raise InstallationError(
                "A atualizacao falhou e o pet original nao pode ser restaurado. "
                f"Backup preservado em {backup}: {restore_exc}"
            ) from install_exc
        raise InstallationError(
            f"A atualizacao falhou; o pet original foi restaurado: {install_exc}"
        ) from install_exc

    _remove_path(backup)
    return destination


def install_copy(
    package: PetPackage,
    pets_root: Path,
    new_display_name: str,
) -> Path:
    display_name = " ".join(new_display_name.split())
    if not display_name:
        raise InstallationError("Informe um novo nome para a copia.")
    new_id = slugify_display_name(display_name)

    pets_root = Path(pets_root)
    destination = pets_root / new_id
    if destination.exists():
        raise InstallationError(f"O pet {new_id} ja existe.")

    staging = _stage_package(package, pets_root)
    manifest_path = staging / "pet.json"
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        manifest["id"] = new_id
        manifest["displayName"] = display_name
        manifest_path.write_text(
            json.dumps(manifest, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
        )
        _rename(staging, destination)
    except (OSError, json.JSONDecodeError, TypeError) as exc:
        _remove_path(staging)
        raise InstallationError(f"Nao foi possivel criar a copia: {exc}") from exc
    return destination

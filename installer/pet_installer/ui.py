from __future__ import annotations

import tkinter as tk
from enum import StrEnum
from pathlib import Path
from tkinter import filedialog, messagebox, simpledialog, ttk

from .comparison import (
    compare_size_maps,
    directory_size_map,
    package_size_map,
)
from .installation import (
    InstallationError,
    default_pets_root,
    install_copy,
    install_new,
    update_existing,
)
from .package import PackageError, PetPackage, validate_pet_zip


class ConflictAction(StrEnum):
    VERIFY = "verify"
    UPDATE = "update"
    COPY = "copy"
    CANCEL = "cancel"


class ConflictDialog:
    def __init__(
        self,
        parent: tk.Misc,
        pet_name: str,
        *,
        allow_verify: bool = True,
    ) -> None:
        self.result = ConflictAction.CANCEL
        self.window = tk.Toplevel(parent)
        self.window.title("Pet já instalado")
        self.window.resizable(False, False)
        self.window.transient(parent)
        self.window.protocol("WM_DELETE_WINDOW", self._cancel)

        frame = ttk.Frame(self.window, padding=20)
        frame.grid(sticky="nsew")
        ttk.Label(
            frame,
            text=f"{pet_name} já está instalado.",
            font=("Segoe UI Semibold", 11),
        ).grid(row=0, column=0, columnspan=2, sticky="w", pady=(0, 6))
        ttk.Label(
            frame,
            text="Escolha como deseja continuar.",
        ).grid(row=1, column=0, columnspan=2, sticky="w", pady=(0, 16))

        row = 2
        if allow_verify:
            ttk.Button(
                frame,
                text="Verificar se é idêntico",
                command=lambda: self._finish(ConflictAction.VERIFY),
            ).grid(row=row, column=0, columnspan=2, sticky="ew", pady=3)
            row += 1
        ttk.Button(
            frame,
            text="Atualizar",
            command=lambda: self._finish(ConflictAction.UPDATE),
        ).grid(row=row, column=0, sticky="ew", padx=(0, 4), pady=3)
        ttk.Button(
            frame,
            text="Instalar como cópia",
            command=lambda: self._finish(ConflictAction.COPY),
        ).grid(row=row, column=1, sticky="ew", padx=(4, 0), pady=3)
        row += 1
        ttk.Button(
            frame,
            text="Cancelar",
            command=self._cancel,
        ).grid(row=row, column=0, columnspan=2, sticky="ew", pady=(8, 0))

        frame.columnconfigure(0, weight=1)
        frame.columnconfigure(1, weight=1)
        self.window.update_idletasks()
        x = parent.winfo_rootx() + max(
            0,
            (parent.winfo_width() - self.window.winfo_width()) // 2,
        )
        y = parent.winfo_rooty() + max(
            0,
            (parent.winfo_height() - self.window.winfo_height()) // 2,
        )
        self.window.geometry(f"+{x}+{y}")
        self.window.grab_set()
        self.window.wait_window()

    def _finish(self, action: ConflictAction) -> None:
        self.result = action
        self.window.destroy()

    def _cancel(self) -> None:
        self._finish(ConflictAction.CANCEL)


class InstallerApp:
    def __init__(self, root: tk.Tk, pets_root: Path | None = None) -> None:
        self.root = root
        self.pets_root = Path(pets_root) if pets_root else default_pets_root()
        self.package: PetPackage | None = None
        self.selected_path: Path | None = None

        root.title("Instalador de Pets do Codex")
        root.geometry("660x390")
        root.minsize(580, 360)

        style = ttk.Style(root)
        if "vista" in style.theme_names():
            style.theme_use("vista")

        container = ttk.Frame(root, padding=24)
        container.grid(row=0, column=0, sticky="nsew")
        root.rowconfigure(0, weight=1)
        root.columnconfigure(0, weight=1)
        container.columnconfigure(0, weight=1)

        ttk.Label(
            container,
            text="Instalador de Pets",
            font=("Segoe UI Semibold", 18),
        ).grid(row=0, column=0, sticky="w")
        ttk.Label(
            container,
            text="Selecione um pacote ZIP baixado do repositório Pets.",
        ).grid(row=1, column=0, sticky="w", pady=(4, 20))

        ttk.Button(
            container,
            text="Selecionar ZIP",
            command=self.select_zip,
        ).grid(row=2, column=0, sticky="w")

        details = ttk.LabelFrame(container, text="Pacote selecionado", padding=14)
        details.grid(row=3, column=0, sticky="nsew", pady=18)
        details.columnconfigure(1, weight=1)
        self.file_value = self._detail_row(details, 0, "Arquivo")
        self.pet_value = self._detail_row(details, 1, "Pet")
        self.id_value = self._detail_row(details, 2, "ID")
        self.destination_value = self._detail_row(details, 3, "Destino")

        self.status_var = tk.StringVar(value="Aguardando seleção de um pacote.")
        ttk.Label(
            container,
            textvariable=self.status_var,
            wraplength=600,
        ).grid(row=4, column=0, sticky="w", pady=(0, 16))

        actions = ttk.Frame(container)
        actions.grid(row=5, column=0, sticky="e")
        ttk.Button(
            actions,
            text="Fechar",
            command=root.destroy,
        ).grid(row=0, column=0, padx=(0, 8))
        self.install_button = ttk.Button(
            actions,
            text="Instalar pet",
            command=self.install_selected,
            state="disabled",
        )
        self.install_button.grid(row=0, column=1)

    def _detail_row(
        self,
        parent: ttk.LabelFrame,
        row: int,
        label: str,
    ) -> ttk.Label:
        ttk.Label(
            parent,
            text=f"{label}:",
            font=("Segoe UI Semibold", 9),
        ).grid(row=row, column=0, sticky="nw", padx=(0, 12), pady=2)
        value = ttk.Label(parent, text="—", wraplength=470)
        value.grid(row=row, column=1, sticky="w", pady=2)
        return value

    def select_zip(self) -> None:
        selected = filedialog.askopenfilename(
            parent=self.root,
            title="Selecionar pacote de pet",
            filetypes=(("Pacotes ZIP", "*.zip"), ("Todos os arquivos", "*.*")),
        )
        if not selected:
            return
        self._load_package(Path(selected))

    def _load_package(self, path: Path) -> None:
        try:
            package = validate_pet_zip(path)
        except PackageError as exc:
            self.package = None
            self.selected_path = None
            self.install_button.configure(state="disabled")
            self.status_var.set("Pacote inválido.")
            messagebox.showerror("Pacote inválido", str(exc), parent=self.root)
            return

        self.package = package
        self.selected_path = path
        self.file_value.configure(text=path.name)
        self.pet_value.configure(text=package.display_name)
        self.id_value.configure(text=package.pet_id)
        self.destination_value.configure(
            text=str(self.pets_root / package.pet_id)
        )
        self.status_var.set("Pacote validado. Pronto para instalar.")
        self.install_button.configure(state="normal")

    def install_selected(self) -> None:
        if self.package is None:
            return
        destination = self.pets_root / self.package.pet_id
        if destination.exists():
            action = ConflictDialog(
                self.root,
                self.package.display_name,
            ).result
            self._handle_conflict(action)
            return
        self._install_new()

    def _install_new(self) -> None:
        assert self.package is not None
        try:
            destination = install_new(self.package, self.pets_root)
        except InstallationError as exc:
            self._show_install_error(exc)
            return
        self._show_success(
            "Pet instalado",
            f"{self.package.display_name} foi instalado em:\n{destination}",
        )

    def _handle_conflict(self, action: ConflictAction) -> None:
        if self.package is None or action is ConflictAction.CANCEL:
            self.status_var.set("Instalação cancelada.")
            return
        if action is ConflictAction.VERIFY:
            self._verify_existing()
        elif action is ConflictAction.UPDATE:
            self._update_existing()
        elif action is ConflictAction.COPY:
            self._install_copy()

    def _verify_existing(self) -> None:
        assert self.package is not None
        messagebox.showwarning(
            "Verificação por tamanho",
            "A verificação compara somente os tamanhos exatos em bits. "
            "Ela não compara o conteúdo dos arquivos.",
            parent=self.root,
        )
        destination = self.pets_root / self.package.pet_id
        comparison = compare_size_maps(
            package_size_map(self.package),
            directory_size_map(destination),
        )
        if comparison.identical:
            self.status_var.set("O pet já está presente e tem os mesmos tamanhos.")
            messagebox.showinfo(
                "Pet já presente",
                "O pet já está presente. A instalação foi cancelada.",
                parent=self.root,
            )
            return

        if comparison.stage == "total":
            detail = (
                "A soma total é diferente:\n"
                f"Pacote: {comparison.incoming_total_bits} bits\n"
                f"Instalado: {comparison.installed_total_bits} bits"
            )
        elif comparison.stage == "paths":
            detail = "A lista de arquivos é diferente."
        else:
            detail = "Um ou mais arquivos têm tamanhos diferentes."
        messagebox.showinfo(
            "Pet distinto",
            f"O pet selecionado é distinto do instalado.\n\n{detail}",
            parent=self.root,
        )
        next_action = ConflictDialog(
            self.root,
            self.package.display_name,
            allow_verify=False,
        ).result
        self._handle_conflict(next_action)

    def _update_existing(self) -> None:
        assert self.package is not None
        if not messagebox.askyesno(
            "Confirmar atualização",
            "Substituir o pet instalado pelo pacote selecionado?",
            parent=self.root,
        ):
            self.status_var.set("Atualização cancelada.")
            return
        try:
            destination = update_existing(self.package, self.pets_root)
        except InstallationError as exc:
            self._show_install_error(exc)
            return
        self._show_success(
            "Pet atualizado",
            f"{self.package.display_name} foi atualizado em:\n{destination}",
        )

    def _install_copy(self) -> None:
        assert self.package is not None
        new_name = simpledialog.askstring(
            "Instalar como cópia",
            "Digite o novo nome do pet:",
            parent=self.root,
        )
        if new_name is None:
            self.status_var.set("Cópia cancelada.")
            return
        try:
            destination = install_copy(
                self.package,
                self.pets_root,
                new_name,
            )
        except InstallationError as exc:
            self._show_install_error(exc)
            return
        self._show_success(
            "Cópia instalada",
            f"A cópia foi instalada em:\n{destination}",
        )

    def _show_install_error(self, error: InstallationError) -> None:
        self.status_var.set("A instalação não foi concluída.")
        messagebox.showerror(
            "Erro de instalação",
            str(error),
            parent=self.root,
        )

    def _show_success(self, title: str, message: str) -> None:
        self.status_var.set(message.replace("\n", " "))
        messagebox.showinfo(title, message, parent=self.root)


def run_app() -> None:
    root = tk.Tk()
    InstallerApp(root)
    root.mainloop()

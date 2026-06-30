from __future__ import annotations

import argparse
import sys
from collections.abc import Sequence


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="Instalador-Pets",
        description="Instalador portatil de pets para o Codex.",
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help=argparse.SUPPRESS,
    )
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    args = _parser().parse_args(argv)
    if args.self_test:
        from .comparison import compare_size_maps
        from .installation import slugify_display_name

        comparison = compare_size_maps({"a": 8}, {"a": 8})
        if not comparison.identical:
            return 1
        if slugify_display_name("Pet Teste") != "pet-teste":
            return 1
        if sys.stdout is not None:
            print("SELF_TEST_OK")
        return 0

    from .ui import run_app

    run_app()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

from __future__ import annotations

import gettext
from contextlib import suppress
from pathlib import Path

from batdetect.pofile import compile_catalogs

DOMAIN = "messages"
LOCALE_DIR = Path(__file__).parent / "locale"
FRENCH = "fr"

with suppress(OSError):
    compile_catalogs(LOCALE_DIR, only_stale=True)
gettext.bindtextdomain(DOMAIN, str(LOCALE_DIR))


def catalog() -> gettext.NullTranslations:
    return gettext.translation(DOMAIN, LOCALE_DIR, fallback=True)


def _(message: str) -> str:
    return catalog().gettext(message)


def ngettext(singular: str, plural: str, count: int) -> str:
    return catalog().ngettext(singular, plural, count)


def is_french() -> bool:
    return catalog().info().get("language") == FRENCH


def decimal(value: float, spec: str = "g") -> str:
    text = format(value, spec)
    return text.replace(".", ",") if is_french() else text

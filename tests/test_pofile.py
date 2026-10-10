from __future__ import annotations

import argparse
import ast
import gettext
import io
import os
import shutil
import subprocess
from pathlib import Path

import pytest

import batdetect
from batdetect.language import LOCALE_DIR
from batdetect.pofile import PLURAL_SEPARATOR, PoError, compile_catalogs, main, mo_bytes, parse_po

FRENCH_PO = LOCALE_DIR / "fr" / "LC_MESSAGES" / "messages.po"
SAMPLE_PO = r"""
msgid ""
msgstr ""
"Content-Type: text/plain; charset=UTF-8\n"
"Language: fr\n"
"Plural-Forms: nplurals=2; plural=(n > 1);\n"

# a comment
msgid "usage: "
msgstr "utilisation : "

msgid "a long "
"message\n"
msgstr "un long "
"message \"cité\"\n"

msgid "{count} track"
msgid_plural "{count} tracks"
msgstr[0] "{count} piste"
msgstr[1] "{count} pistes"
"""


def loaded(mo: bytes) -> gettext.GNUTranslations:
    return gettext.GNUTranslations(io.BytesIO(mo))


def messages_called_in(source: Path) -> set[str]:
    found: set[str] = set()
    for node in ast.walk(ast.parse(source.read_text(encoding="utf-8"))):
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Name) and node.func.id in {"_", "ngettext"}:
            strings = [arg.value for arg in node.args if isinstance(arg, ast.Constant) and isinstance(arg.value, str)]
            found.add(PLURAL_SEPARATOR.join(strings))
    return found


def test_compiled_sample_translates_singular_plural_and_continued_lines() -> None:
    catalog = loaded(mo_bytes(parse_po(SAMPLE_PO)))

    assert catalog.gettext("usage: ") == "utilisation : "
    assert catalog.gettext("a long message\n") == 'un long message "cité"\n'
    assert catalog.ngettext("{count} track", "{count} tracks", 1) == "{count} piste"
    assert catalog.ngettext("{count} track", "{count} tracks", 2) == "{count} pistes"
    assert catalog.info()["language"] == "fr"


def test_compiled_french_catalog_translates_the_messages_of_the_command(tmp_path: Path) -> None:
    po = tmp_path / "fr" / "LC_MESSAGES" / "messages.po"
    po.parent.mkdir(parents=True)
    shutil.copy(FRENCH_PO, po)
    compile_catalogs(tmp_path)
    catalog = loaded(po.with_suffix(".mo").read_bytes())

    assert catalog.gettext("no video found") == "aucune vidéo trouvée"
    assert catalog.ngettext("{count} track", "{count} tracks", 2) == "{count} pistes"
    assert catalog.info()["language"] == "fr"


def test_only_missing_or_older_catalogs_are_compiled_again(tmp_path: Path) -> None:
    po = tmp_path / "fr" / "LC_MESSAGES" / "messages.po"
    po.parent.mkdir(parents=True)
    po.write_text(SAMPLE_PO, encoding="utf-8")
    mo = po.with_suffix(".mo")

    assert compile_catalogs(tmp_path, only_stale=True) == [mo]
    assert compile_catalogs(tmp_path, only_stale=True) == []
    os.utime(mo, (0, 0))
    assert compile_catalogs(tmp_path, only_stale=True) == [mo]
    assert [p.name for p in po.parent.iterdir() if p.suffix == ".tmp"] == []


def test_command_line_compiles_every_catalog_under_the_given_folder(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    po = tmp_path / "fr" / "LC_MESSAGES" / "messages.po"
    po.parent.mkdir(parents=True)
    po.write_text(SAMPLE_PO, encoding="utf-8")

    assert main([str(tmp_path)]) == 0
    assert capsys.readouterr().out.strip() == str(po.with_suffix(".mo"))


@pytest.mark.skipif(shutil.which("msgfmt") is None, reason="msgfmt only serves as a cross-check")
def test_catalog_reads_the_same_as_the_one_gnu_msgfmt_writes(tmp_path: Path) -> None:
    reference = tmp_path / "messages.mo"
    subprocess.run(["msgfmt", "-o", str(reference), str(FRENCH_PO)], check=True)
    ours = loaded(mo_bytes(parse_po(FRENCH_PO.read_text(encoding="utf-8"))))
    theirs = loaded(reference.read_bytes())

    for key in parse_po(FRENCH_PO.read_text(encoding="utf-8")):
        if PLURAL_SEPARATOR in key:
            singular, plural = key.split(PLURAL_SEPARATOR)
            for n in (0, 1, 2):
                assert ours.ngettext(singular, plural, n) == theirs.ngettext(singular, plural, n)
        else:
            assert ours.gettext(key) == theirs.gettext(key)


def test_every_message_of_the_code_and_of_argparse_has_a_french_translation() -> None:
    catalog = parse_po(FRENCH_PO.read_text(encoding="utf-8"))
    sources = [*Path(batdetect.__file__).parent.rglob("*.py"), Path(argparse.__file__)]
    used = set[str]().union(*(messages_called_in(source) for source in sources))

    assert sorted(used - catalog.keys()) == []
    assert [key for key, value in catalog.items() if key and not value] == []


@pytest.mark.parametrize(
    "text",
    [
        'msgid "a"\nmsgctxt "b"\n',
        '"orphan"\n',
        "msgid unquoted\n",
        'msgid "a"\nmsgstr "b" trailing"\n',
        'msgid "a"\n',
        'msgid "a"\nmsgstr "b"\nmsgstr "c"\n',
        'msgid "a"\nmsgstr "b"\n\nmsgid "a"\nmsgstr "c"\n',
        'msgid "a"\nmsgid_plural "as"\nmsgstr[0] "b"\n',
    ],
)
def test_malformed_po_is_refused_instead_of_dropping_a_message(text: str) -> None:
    with pytest.raises(PoError):
        parse_po(text)


def test_untranslated_and_fuzzy_entries_are_left_out_so_the_english_text_shows_instead_of_nothing() -> None:
    catalog = parse_po('msgid "a"\nmsgstr ""\n\n#, fuzzy\nmsgid "b"\nmsgstr "bé"\n\nmsgid "c"\nmsgstr "cé"\n')

    assert catalog == {"c": "cé"}

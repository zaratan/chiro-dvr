from __future__ import annotations

import argparse

import pytest

from batdetect.language import _, decimal, is_french, ngettext


def test_english_when_no_locale_variable_is_set() -> None:
    assert not is_french()
    assert _("no video found") == "no video found"


@pytest.mark.parametrize("value", ["fr_FR.UTF-8", "fr_CA.UTF-8", "fr"])
def test_french_when_lang_names_a_french_locale(monkeypatch: pytest.MonkeyPatch, value: str) -> None:
    monkeypatch.setenv("LANG", value)

    assert is_french()
    assert _("no video found") == "aucune vidéo trouvée"


@pytest.mark.parametrize("value", ["C", "en_US.UTF-8", "de_DE.UTF-8"])
def test_english_for_any_other_locale(monkeypatch: pytest.MonkeyPatch, value: str) -> None:
    monkeypatch.setenv("LANG", value)

    assert not is_french()


def test_lc_all_wins_over_lang_both_ways(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("LANG", "fr_FR.UTF-8")
    monkeypatch.setenv("LC_ALL", "C")
    assert not is_french()

    monkeypatch.setenv("LANG", "C")
    monkeypatch.setenv("LC_ALL", "fr_FR.UTF-8")
    assert is_french()


def test_lc_messages_wins_over_lang_but_not_over_lc_all(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("LANG", "C")
    monkeypatch.setenv("LC_MESSAGES", "fr_FR.UTF-8")
    assert is_french()

    monkeypatch.setenv("LC_ALL", "en_US.UTF-8")
    assert not is_french()


def test_empty_variable_is_skipped_like_gettext_does(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("LC_ALL", "")
    monkeypatch.setenv("LANG", "fr_FR.UTF-8")

    assert is_french()


@pytest.mark.usefixtures("french")
def test_french_singular_covers_zero_and_one() -> None:
    assert [ngettext("{count} track", "{count} tracks", n).format(count=n) for n in (0, 1, 2)] == [
        "0 piste",
        "1 piste",
        "2 pistes",
    ]


def test_english_plural_starts_at_zero() -> None:
    assert [ngettext("{count} track", "{count} tracks", n).format(count=n) for n in (0, 1, 2)] == [
        "0 tracks",
        "1 track",
        "2 tracks",
    ]


@pytest.mark.usefixtures("french")
def test_french_numbers_take_a_decimal_comma_without_touching_the_process_locale() -> None:
    assert decimal(1.5) == "1,5"
    assert decimal(2.34, ".1f") == "2,3"
    assert decimal(2700) == "2700"


def test_english_numbers_keep_the_decimal_point() -> None:
    assert decimal(1.5) == "1.5"
    assert decimal(2.34, ".1f") == "2.3"


@pytest.mark.usefixtures("french")
def test_a_bare_parser_built_before_any_message_is_translated_already_speaks_french() -> None:
    assert "affiche ce message d'aide et quitte" in argparse.ArgumentParser(prog="p").format_help()

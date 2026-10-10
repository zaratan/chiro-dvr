from __future__ import annotations

import ast
import struct
import sys
import tempfile
from pathlib import Path

MO_MAGIC = 0x950412DE
MO_HEADER_SIZE = 7 * 4
PLURAL_SEPARATOR = "\0"
PLURAL_FORMS = 2


class PoError(ValueError):
    pass


def _quoted(line: str, number: int) -> str:
    if not (line.startswith('"') and line.endswith('"')):
        raise PoError(f"line {number}: expected a quoted string")
    try:
        value = ast.literal_eval(line)
    except (SyntaxError, ValueError) as err:
        raise PoError(f"line {number}: malformed string") from err
    if not isinstance(value, str):
        raise PoError(f"line {number}: expected a quoted string")
    return value


def _entries(text: str) -> list[tuple[bool, dict[str, str]]]:
    entries: list[tuple[bool, dict[str, str]]] = []
    current: dict[str, str] = {}
    fuzzy = marked_fuzzy = False
    field = ""
    for number, raw in enumerate(text.splitlines(), start=1):
        line = raw.strip()
        if not line:
            continue
        if line.startswith("#"):
            marked_fuzzy = marked_fuzzy or (line.startswith("#,") and "fuzzy" in line)
            continue
        if line.startswith('"'):
            if not field:
                raise PoError(f"line {number}: string outside an entry")
            current[field] += _quoted(line, number)
            continue
        keyword, _, rest = line.partition(" ")
        if keyword == "msgid":
            if current:
                entries.append((fuzzy, current))
            current, fuzzy, marked_fuzzy = {}, marked_fuzzy, False
        if keyword not in {"msgid", "msgid_plural", "msgstr"} and not keyword.startswith("msgstr["):
            raise PoError(f"line {number}: unknown keyword {keyword}")
        if keyword in current:
            raise PoError(f"line {number}: {keyword} given twice")
        field = keyword
        current[field] = _quoted(rest.strip(), number)
    if current:
        entries.append((fuzzy, current))
    return entries


def _translation(entry: dict[str, str]) -> tuple[str, str]:
    if "msgid" not in entry:
        raise PoError("translation without msgid")
    if "msgid_plural" not in entry:
        if "msgstr" not in entry:
            raise PoError(f"no msgstr for {entry['msgid']!r}")
        return entry["msgid"], entry["msgstr"]
    forms = [entry.get(f"msgstr[{index}]") for index in range(PLURAL_FORMS)]
    if any(form is None for form in forms) or len(entry) != 2 + PLURAL_FORMS:
        raise PoError(f"expected msgstr[0] and msgstr[1] for {entry['msgid']!r}")
    return entry["msgid"] + PLURAL_SEPARATOR + entry["msgid_plural"], PLURAL_SEPARATOR.join(map(str, forms))


def parse_po(text: str) -> dict[str, str]:
    messages: dict[str, str] = {}
    for fuzzy, entry in _entries(text):
        key, value = _translation(entry)
        if key in messages:
            raise PoError(f"{key!r} translated twice")
        if not fuzzy and value.strip(PLURAL_SEPARATOR):
            messages[key] = value
    return messages


def mo_bytes(messages: dict[str, str]) -> bytes:
    keys = sorted(messages)
    ids = [key.encode() for key in keys]
    strs = [messages[key].encode() for key in keys]
    count = len(keys)
    ids_table = MO_HEADER_SIZE
    strs_table = ids_table + 8 * count
    data_start = strs_table + 8 * count
    table: list[int] = []
    data = b""
    for chunk in (*ids, *strs):
        table += [len(chunk), data_start + len(data)]
        data += chunk + b"\0"
    header = struct.pack("<7I", MO_MAGIC, 0, count, ids_table, strs_table, 0, data_start)
    return header + struct.pack(f"<{4 * count}I", *table) + data


def is_stale(po: Path) -> bool:
    mo = po.with_suffix(".mo")
    return not mo.exists() or mo.stat().st_mtime < po.stat().st_mtime


def write_mo(po: Path) -> Path:
    mo = po.with_suffix(".mo")
    data = mo_bytes(parse_po(po.read_text(encoding="utf-8")))
    with tempfile.NamedTemporaryFile(dir=po.parent, suffix=".tmp", delete=False) as handle:
        handle.write(data)
    try:
        Path(handle.name).replace(mo)
    finally:
        Path(handle.name).unlink(missing_ok=True)
    return mo


def compile_catalogs(root: Path, *, only_stale: bool = False) -> list[Path]:
    return [write_mo(po) for po in sorted(root.glob("*/LC_MESSAGES/*.po")) if not only_stale or is_stale(po)]


def main(argv: list[str] | None = None) -> int:
    for root in argv if argv is not None else sys.argv[1:]:
        for mo in compile_catalogs(Path(root)):
            print(mo)
    return 0


if __name__ == "__main__":
    sys.exit(main())

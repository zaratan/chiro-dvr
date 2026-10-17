"use strict";
const VerdictsCsv = (() => {
  const SEPARATORS = [";", ",", "\t"];
  const QUOTE = '"';
  const BYTE_ORDER_MARK = String.fromCharCode(0xfeff);
  const LINE_END = "\r\n";
  const COLUMN_COUNT = 8;
  const [VIDEO, TRACK, START, START_SECONDS, VERDICT, SPECIES, START_X, START_Y] = Array.from({ length: COLUMN_COUNT }, (_, index) => index);

  const decode = (bytes) => {
    try {
      return new TextDecoder("utf-8", { fatal: true }).decode(bytes);
    } catch {
      return new TextDecoder("windows-1252").decode(bytes);
    }
  };

  const folded = (value) => value.toLowerCase().replace(/[^a-z0-9]/g, "");

  const separatorOf = (text) => {
    const line = text.split(/\r?\n/, 1)[0];
    const count = (separator) => line.split(separator).length;
    return SEPARATORS.reduce((best, separator) => (count(separator) > count(best) ? separator : best));
  };

  const records = (text, separator) => {
    const rows = [];
    let row = [];
    let cell = "";
    let quoted = false;
    for (let index = 0; index < text.length; index += 1) {
      const character = text[index];
      if (quoted) {
        if (character === QUOTE && text[index + 1] === QUOTE) {
          cell += QUOTE;
          index += 1;
        } else if (character === QUOTE) {
          quoted = false;
        } else {
          cell += character;
        }
      } else if (character === QUOTE) {
        quoted = true;
      } else if (character === separator) {
        row.push(cell);
        cell = "";
      } else if (character === "\n" || character === "\r") {
        if (character === "\r" && text[index + 1] === "\n") index += 1;
        row.push(cell);
        rows.push(row);
        row = [];
        cell = "";
      } else {
        cell += character;
      }
    }
    if (cell !== "" || row.length > 0) {
      row.push(cell);
      rows.push(row);
    }
    return rows.filter((one) => one.some((value) => value.trim() !== ""));
  };

  const number = (value) => {
    const parsed = Number(value.replace(",", "."));
    return value !== "" && Number.isFinite(parsed) ? parsed : null;
  };

  const read = (text, headings) => {
    const clean = text.startsWith(BYTE_ORDER_MARK) ? text.slice(1) : text;
    const [header = [], ...body] = records(clean, separatorOf(clean));
    let positions = headings.map((heading) => header.findIndex((cell) => folded(cell) === folded(heading)));
    if (positions[VIDEO] < 0 || positions[START_SECONDS] < 0) {
      if (header.length !== COLUMN_COUNT) return null;
      positions = headings.map((_, index) => index);
    }
    return body
      .map((cells) => {
        const cell = (column) => (positions[column] >= 0 ? (cells[positions[column]] ?? "").trim() : "");
        return {
          video: cell(VIDEO),
          track: cell(TRACK),
          start: cell(START),
          startSeconds: number(cell(START_SECONDS)),
          verdict: cell(VERDICT),
          species: cell(SPECIES),
          startX: number(cell(START_X)),
          startY: number(cell(START_Y)),
        };
      })
      .filter((row) => Object.values(row).some((value) => value !== "" && value !== null));
  };

  const write = (rows, { headings, separator, decimal }) => {
    const quoted = (value) => {
      const text = String(value);
      return [separator, QUOTE, "\r", "\n"].some((special) => text.includes(special))
        ? QUOTE + text.replaceAll(QUOTE, QUOTE + QUOTE) + QUOTE
        : text;
    };
    const line = (cells) => cells.map(quoted).join(separator) + LINE_END;
    const seconds = (value) => (value === null ? "" : value.toFixed(2).replace(".", decimal));
    const coordinate = (value) => (value === null || value === undefined ? "" : String(value));
    return (
      BYTE_ORDER_MARK +
      line(headings) +
      rows
        .map((row) =>
          line([
            row.video,
            row.track,
            row.start,
            seconds(row.startSeconds),
            row.verdict,
            row.species,
            coordinate(row.startX),
            coordinate(row.startY),
          ]),
        )
        .join("")
    );
  };

  const FNV_OFFSET = 0x811c9dc5;
  const FNV_PRIME = 0x01000193;
  const fingerprint = (bytes) =>
    bytes.reduce((hash, byte) => Math.imul(hash ^ byte, FNV_PRIME) >>> 0, FNV_OFFSET).toString(16);

  return { decode, read, write, fingerprint };
})();

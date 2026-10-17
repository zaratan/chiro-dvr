import { test } from "node:test";
import assert from "node:assert/strict";
import { load } from "./load.mjs";

const { VerdictsCsv } = load("csv.js");
const FRENCH = ["vidéo", "piste", "début", "début (s)", "verdict", "espèce", "départ x (px)", "départ y (px)"];
const ENGLISH = ["video", "track", "start", "start (s)", "verdict", "species", "start x (px)", "start y (px)"];
const row = {
  video: "video_092_original",
  track: "5",
  start: "0:01:12",
  startSeconds: 72.39,
  verdict: "oui",
  species: "Pipistrelle commune; ou \"kuhl\"",
  startX: 959,
  startY: 644,
};

test("a file written in french reads back the same rows", () => {
  const text = VerdictsCsv.write([row], { headings: FRENCH, separator: ";", decimal: "," });
  assert.ok(text.startsWith(`${String.fromCharCode(0xfeff)}vidéo;piste;début;début (s);`));
  assert.ok(text.includes(";72,39;"));
  assert.deepEqual(VerdictsCsv.read(text, FRENCH), [row]);
});

test("a file written in english with points reads back the same rows", () => {
  const text = VerdictsCsv.write([row], { headings: ENGLISH, separator: ",", decimal: "." });
  assert.deepEqual(VerdictsCsv.read(text, ENGLISH), [row]);
});

test("bytes in utf-8 or saved by a spreadsheet in windows-1252 decode to the same accents", () => {
  const utf8 = new TextEncoder().encode("vidéo;espèce");
  const latin = Uint8Array.from([0x76, 0x69, 0x64, 0xe9, 0x6f, 0x3b, 0x65, 0x73, 0x70, 0xe8, 0x63, 0x65]);
  assert.equal(VerdictsCsv.decode(utf8), "vidéo;espèce");
  assert.equal(VerdictsCsv.decode(latin), "vidéo;espèce");
});

test("a table resaved by a spreadsheet with tabs, other cases and moved columns is still read", () => {
  const text = "Verdict\tVIDÉO\tDébut (s)\tEspèce\r\noui\tv\t4.8\tmurin\r\n";
  assert.deepEqual(VerdictsCsv.read(text, FRENCH), [
    { video: "v", track: "", start: "", startSeconds: 4.8, verdict: "oui", species: "murin", startX: null, startY: null },
  ]);
});

test("a heading badly decoded by another encoding is still recognised", () => {
  const text = "vidƒo;piste;dƒbut;dƒbut (s);verdict\r\nv;1;0:00:04;4,8;non\r\n";
  assert.equal(VerdictsCsv.read(text, FRENCH)[0].verdict, "non");
});

test("a file from a page in the other language is read by the order of its columns", () => {
  const text = VerdictsCsv.write([row], { headings: ENGLISH, separator: ",", decimal: "." });
  assert.deepEqual(VerdictsCsv.read(text, FRENCH), [row]);
});

test("a file that is not a verdicts file reads as nothing", () => {
  assert.equal(VerdictsCsv.read("a;b\r\n1;2\r\n", FRENCH), null);
});

test("a line with an unreadable start is kept so that it is written back", () => {
  const text = "vidéo;début (s);verdict\r\nv;;oui\r\nv;4,8;oui\r\n;;\r\n";
  assert.deepEqual(VerdictsCsv.read(text, FRENCH).map((one) => one.startSeconds), [null, 4.8]);
});

test("the fingerprint of a file changes with its content, not only its size", () => {
  const encoder = new TextEncoder();
  assert.notEqual(VerdictsCsv.fingerprint(encoder.encode("v;oui")), VerdictsCsv.fingerprint(encoder.encode("v;non")));
  assert.equal(VerdictsCsv.fingerprint(encoder.encode("v;oui")), VerdictsCsv.fingerprint(encoder.encode("v;oui")));
});

import http from "node:http";
import fs from "node:fs";
import path from "node:path";
const [root, port, isolated] = [process.argv[2], Number(process.argv[3]), process.argv[4] === "isolated"];
const types = { ".html": "text/html", ".js": "text/javascript", ".mjs": "text/javascript", ".wasm": "application/wasm", ".mp4": "video/mp4", ".json": "application/json", ".csv": "text/csv", ".png": "image/png" };
http.createServer((request, response) => {
  const file = path.join(root, decodeURIComponent(new URL(request.url, "http://x").pathname));
  fs.stat(file, (error, stat) => {
    if (error || !stat.isFile()) { response.writeHead(404); response.end(); return; }
    const headers = { "Content-Type": types[path.extname(file)] ?? "application/octet-stream", "Content-Length": stat.size };
    if (isolated) Object.assign(headers, { "Cross-Origin-Opener-Policy": "same-origin", "Cross-Origin-Embedder-Policy": "require-corp" });
    response.writeHead(200, headers);
    fs.createReadStream(file).pipe(response);
  });
}).listen(port, "127.0.0.1");

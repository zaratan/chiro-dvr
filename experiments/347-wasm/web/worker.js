import { Input, BlobSource, ALL_FORMATS, EncodedPacketSink } from "./node_modules/mediabunny/dist/bundles/mediabunny.min.mjs";

function hashOf(bytes) {
  const words = new Uint32Array(bytes.buffer, bytes.byteOffset, bytes.byteLength >>> 2);
  let hash = 2166136261;
  for (let i = 0; i < words.length; i++) hash = Math.imul(hash ^ words[i], 16777619) >>> 0;
  return hash;
}

onmessage = async ({ data: options }) => {
  try {
    const started = performance.now();
    let wasm = null, session = null, memory = null;
    if (options.detect) {
      wasm = await import(options.pkg + "/webproto.js");
      const exports = await wasm.default();
      memory = exports.memory;
      if (options.threads) { console.log("pool start"); await wasm.initThreadPool(options.threads); console.log("pool ready"); }
    }
    const blob = await (await fetch(options.video)).blob();
    const input = new Input({ source: new BlobSource(blob), formats: ALL_FORMATS });
    const track = await input.getPrimaryVideoTrack();
    const config = await track.getDecoderConfig();
    config.hardwareAcceleration = options.acceleration ?? "no-preference";
    const support = await VideoDecoder.isConfigSupported(config);
    const width = track.codedWidth, height = track.codedHeight;
    const announced = options.limit ?? (await track.computePacketStats()).packetCount;
    if (options.detect) { session = new wasm.WebSession(width, height, 10000, 333, announced); console.log("session workers " + session.workers()); }
    const hashes = [], formats = new Set(), timestamps = [];
    let pending = Promise.resolve(), decodedAt = 0, detectMs = 0, peakMemory = 0;
    const buffer = new Uint8Array(width * height * 3 / 2);
    const decoder = new VideoDecoder({
      output: (frame) => {
        formats.add(frame.format + " " + frame.colorSpace?.matrix + " " + frame.colorSpace?.fullRange);
        timestamps.push(frame.timestamp);
        pending = pending.then(async () => {
          try {
          if (frame.format !== "I420" || frame.allocationSize() !== buffer.byteLength) throw new Error("unexpected " + frame.format + " " + frame.allocationSize());
          await frame.copyTo(buffer);
          frame.close();
          if (options.hash) hashes.push(hashOf(buffer));
          if (session) {
            const t = performance.now();
            const luma = width * height, chroma = luma / 4;
            session.push(buffer.subarray(0, luma), buffer.subarray(luma, luma + chroma), buffer.subarray(luma + chroma));
            detectMs += performance.now() - t;
            if (options.trace && timestamps.length % 32 === 0) console.log("pushed " + timestamps.length);
            peakMemory = Math.max(peakMemory, memory.buffer.byteLength);
          }
          } catch (error) { postMessage({ error: String(error) }); }
        });
      },
      error: (error) => postMessage({ error: String(error) }),
    });
    decoder.configure(config);
    const sink = new EncodedPacketSink(track);
    let count = 0;
    for await (const packet of sink.packets()) {
      if (options.limit && count >= options.limit) break;
      while (decoder.decodeQueueSize > 8) await new Promise((r) => decoder.addEventListener("dequeue", r, { once: true }));
      decoder.decode(packet.toEncodedVideoChunk());
      count++;
      if (count % 64 === 0) await pending;
    }
    await decoder.flush();
    await pending;
    decodedAt = performance.now();
    let result = "", workers = 0;
    if (session) {
      workers = session.workers();
      const t = performance.now();
      result = session.finish();
      detectMs += performance.now() - t;
      peakMemory = Math.max(peakMemory, memory.buffer.byteLength);
    }
    postMessage({ done: true, frames: timestamps.length, packets: count, width, height, codec: config.codec, supported: support.supported,
      formats: [...formats], firstTimestamps: timestamps.slice(0, 4), hashes, result, workers, detectMs, peakMemory,
      totalMs: performance.now() - started, decodedMs: decodedAt - started });
  } catch (error) {
    postMessage({ error: String(error?.stack ?? error) });
  }
};

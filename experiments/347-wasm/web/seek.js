import { Input, BlobSource, ALL_FORMATS, EncodedPacketSink } from "./node_modules/mediabunny/dist/bundles/mediabunny.min.mjs";

function hashOf(bytes) {
  const words = new Uint32Array(bytes.buffer, bytes.byteOffset, bytes.byteLength >>> 2);
  let hash = 2166136261;
  for (let i = 0; i < words.length; i++) hash = Math.imul(hash ^ words[i], 16777619) >>> 0;
  return hash;
}

onmessage = async ({ data: options }) => {
  try {
    const blob = await (await fetch(options.video)).blob();
    const input = new Input({ source: new BlobSource(blob), formats: ALL_FORMATS });
    const track = await input.getPrimaryVideoTrack();
    const config = await track.getDecoderConfig();
    const sink = new EncodedPacketSink(track);
    const results = [];
    for (const target of options.targets) {
      const t0 = performance.now();
      const wanted = target * 333 / 10000;
      const key = await sink.getKeyPacket(wanted + 1e-6);
      const last = await sink.getPacket(wanted + 1e-6);
      let decodedCount = 0, found = null;
      const decoder = new VideoDecoder({
        output: async (frame) => { decodedCount++; if (Math.abs(frame.timestamp - last.timestamp * 1e6) < 1) { const b = new Uint8Array(frame.allocationSize()); await frame.copyTo(b); found = { hash: hashOf(b), timestamp: frame.timestamp }; } frame.close(); },
        error: (e) => postMessage({ error: String(e) }),
      });
      decoder.configure(config);
      for await (const packet of sink.packets(key, (await sink.getNextPacket(last)) ?? undefined)) decoder.decode(packet.toEncodedVideoChunk());
      await decoder.flush();
      decoder.close();
      results.push({ target, keyTimestamp: key.timestamp, decodedCount, hash: found?.hash, ms: performance.now() - t0 });
    }
    postMessage({ done: true, results });
  } catch (error) {
    postMessage({ error: String(error?.stack ?? error) });
  }
};

import { Input, BlobSource, ALL_FORMATS, EncodedPacketSink, Output, Mp4OutputFormat, BufferTarget, EncodedVideoPacketSource, EncodedPacket } from "./node_modules/mediabunny/dist/bundles/mediabunny.min.mjs";

const SUPPORT = [];
async function supportMatrix(width, height) {
  for (const codec of ["avc1.42001f", "avc1.42002a", "avc1.4d0028", "avc1.640028", "avc1.640032", "hvc1.1.6.L123.B0", "vp09.00.40.08", "av01.0.08M.08"]) {
    for (const hardwareAcceleration of ["prefer-software", "prefer-hardware"]) {
      for (const bitrateMode of ["quantizer", "variable"]) {
        const config = { codec, width, height, framerate: 30.03, hardwareAcceleration, bitrateMode, bitrate: 24_000_000, avc: { format: "avc" } };
        let supported = false;
        try { supported = (await VideoEncoder.isConfigSupported(config)).supported; } catch (error) { supported = "error " + error.name; }
        SUPPORT.push({ codec, hardwareAcceleration, bitrateMode, supported });
      }
    }
  }
}

onmessage = async ({ data: options }) => {
  try {
    const blob = await (await fetch(options.video)).blob();
    const input = new Input({ source: new BlobSource(blob), formats: ALL_FORMATS });
    const track = await input.getPrimaryVideoTrack();
    const width = track.codedWidth, height = track.codedHeight;
    await supportMatrix(width, height);
    const decoder = new VideoDecoder({ output: (frame) => frames.push(frame), error: (e) => postMessage({ error: String(e) }) });
    const frames = [];
    decoder.configure(await track.getDecoderConfig());
    const wanted = options.clipFrames;
    let count = 0;
    for await (const packet of new EncodedPacketSink(track).packets()) {
      if (count++ >= wanted) break;
      decoder.decode(packet.toEncodedVideoChunk());
    }
    await decoder.flush();
    const buffers = [];
    for (const frame of frames) {
      const buffer = new Uint8Array(frame.allocationSize());
      await frame.copyTo(buffer);
      buffers.push({ buffer, timestamp: frame.timestamp, duration: frame.duration });
      frame.close();
    }
    const results = [];
    const started = performance.now();
    const one = async (clip) => {
      const output = new Output({ format: new Mp4OutputFormat({ fastStart: "in-memory" }), target: new BufferTarget() });
      const source = new EncodedVideoPacketSource("avc");
      output.addVideoTrack(source, { frameRate: 10000 / 333 });
      await output.start();
      let chain = Promise.resolve(), encoded = 0, encodedBytes = 0;
      const encoder = new VideoEncoder({
        output: (chunk, meta) => { encoded++; encodedBytes += chunk.byteLength; const packet = EncodedPacket.fromEncodedChunk(chunk); chain = chain.then(() => source.add(packet, meta)); },
        error: (e) => postMessage({ error: String(e) }),
      });
      const config = { codec: options.codec, width, height, framerate: 10000 / 333, hardwareAcceleration: options.acceleration, bitrateMode: options.bitrateMode, bitrate: options.bitrate, latencyMode: "quality", avc: { format: "avc" } };
      encoder.configure(config);
      const t0 = performance.now();
      for (const [index, { buffer, timestamp, duration }] of buffers.entries()) {
        const drawn = buffer.slice();
        const box = 200 + (index % 50) * 4;
        for (let y = 300; y < 340; y++) { const row = y * width; drawn.fill(235, row + box, row + box + 2); drawn.fill(235, row + box + 38, row + box + 40); }
        for (const y of [300, 301, 338, 339]) drawn.fill(235, y * width + box, y * width + box + 40);
        const frame = new VideoFrame(drawn, { format: "I420", codedWidth: width, codedHeight: height, timestamp: timestamp - buffers[0].timestamp, duration, colorSpace: { primaries: "bt709", transfer: "bt709", matrix: "bt709", fullRange: false } });
        while (encoder.encodeQueueSize > 4) await new Promise((r) => encoder.addEventListener("dequeue", r, { once: true }));
        const encodeOptions = { keyFrame: index === 0 };
        if (options.bitrateMode === "quantizer") encodeOptions.avc = { quantizer: options.quantizer };
        encoder.encode(frame, encodeOptions);
        frame.close();
      }
      await encoder.flush();
      const encodeMs = performance.now() - t0;
      encoder.close();
      await chain;
      await output.finalize();
      const bytes = output.target.buffer;
      results.push({ clip, frames: buffers.length, encoded, encodedBytes, mp4Bytes: bytes.byteLength, encodeMs });
      if (clip === 0) postMessage({ file: bytes }, [bytes]);
    };
    if (options.parallel) await Promise.all([...Array(options.clips).keys()].map(one)); else for (let clip = 0; clip < options.clips; clip++) await one(clip);
    postMessage({ done: true, width, height, support: SUPPORT, results, totalMs: performance.now() - started });
  } catch (error) {
    postMessage({ error: String(error?.stack ?? error) });
  }
};

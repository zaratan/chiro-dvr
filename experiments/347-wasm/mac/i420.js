let scratch = new Uint8Array(0);

export async function copyI420(frame, buffer) {
  const size = frame.allocationSize();
  if (size !== buffer.byteLength) throw new Error("unexpected size " + frame.format + " " + size);
  if (frame.format === "I420") { await frame.copyTo(buffer); return; }
  if (frame.format !== "NV12") throw new Error("unexpected format " + frame.format);
  if (scratch.byteLength !== size) scratch = new Uint8Array(size);
  await frame.copyTo(scratch);
  const luma = frame.visibleRect.width * frame.visibleRect.height, chroma = luma / 4;
  buffer.set(scratch.subarray(0, luma));
  for (let i = 0; i < chroma; i++) {
    buffer[luma + i] = scratch[luma + 2 * i];
    buffer[luma + chroma + i] = scratch[luma + 2 * i + 1];
  }
}

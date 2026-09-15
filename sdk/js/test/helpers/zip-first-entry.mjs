/** Read the first local ZIP entry (name, compression method, uncompressed body). */

const SIG = 0x04034b50;

export function zipFirstEntry(bytes) {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  if (view.getUint32(0, true) !== SIG) {
    throw new Error("not a ZIP local file header");
  }
  const method = view.getUint16(8, true);
  const nameLen = view.getUint16(26, true);
  const extraLen = view.getUint16(28, true);
  const nameStart = 30;
  const name = new TextDecoder().decode(bytes.subarray(nameStart, nameStart + nameLen));
  let off = nameStart + nameLen + extraLen;
  const compSize = view.getUint32(18, true);
  const uncompSize = view.getUint32(22, true);
  const payload = bytes.subarray(off, off + compSize);
  if (method === 0) {
    return { name, compression: "stored", data: payload, uncompressedSize: uncompSize };
  }
  return { name, compression: "deflated", data: payload, uncompressedSize: uncompSize };
}

type TensorMeta = { shape: number[] }
type NtfHeader = { tensors: TensorMeta[] }

const ASCII = new TextDecoder()

function product(values: number[]) {
  return values.reduce((result, value) => result * value, 1)
}

function weightCount(header: NtfHeader) {
  return header.tensors.reduce(
    (count, tensor) => count + product(tensor.shape),
    0,
  )
}

function halfToFloat(value: number) {
  const sign = value & 0x8000 ? -1 : 1
  const exponent = (value >> 10) & 0x1f
  const fraction = value & 0x03ff
  if (exponent === 0) return sign * 2 ** -14 * (fraction / 1024)
  if (exponent === 0x1f)
    return fraction ? Number.NaN : sign * Number.POSITIVE_INFINITY
  return sign * 2 ** (exponent - 15) * (1 + fraction / 1024)
}

function expandF16(bytes: Uint8Array): Uint8Array {
  const source = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength)
  const headerLength = source.getUint32(4, true)
  const headerStart = 8
  const headerEnd = headerStart + headerLength
  const header = JSON.parse(
    ASCII.decode(bytes.subarray(headerStart, headerEnd)),
  ) as NtfHeader
  const count = weightCount(header)
  if (headerEnd + count * 2 > bytes.length) throw new Error('truncated f16 NTF')

  const expanded = new Uint8Array(8 + headerLength + count * 4)
  expanded.set([0x4e, 0x54, 0x46, 0x30]) // NTF0
  expanded.set(bytes.subarray(4, headerEnd), 4)
  const output = new DataView(expanded.buffer)
  for (let i = 0; i < count; i++) {
    output.setFloat32(
      8 + headerLength + i * 4,
      halfToFloat(source.getUint16(headerEnd + i * 2, true)),
      true,
    )
  }
  return expanded
}

function expandInt8(bytes: Uint8Array): Uint8Array {
  const source = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength)
  const headerLength = source.getUint32(4, true)
  const headerStart = 8
  const headerEnd = headerStart + headerLength
  const header = JSON.parse(
    ASCII.decode(bytes.subarray(headerStart, headerEnd)),
  ) as NtfHeader
  const blockSize = source.getUint32(headerEnd, true)
  const weightCount = source.getUint32(headerEnd + 4, true)
  const tensorCounts = header.tensors.map((tensor) => product(tensor.shape))
  const scaleCount = tensorCounts.reduce(
    (count, tensorCount) => count + Math.ceil(tensorCount / blockSize),
    0,
  )
  const scalesStart = headerEnd + 8
  const weightsStart = scalesStart + scaleCount * 4
  if (weightsStart + weightCount > bytes.length)
    throw new Error('truncated int8 NTF')

  const expanded = new Uint8Array(8 + headerLength + weightCount * 4)
  expanded.set([0x4e, 0x54, 0x46, 0x30]) // NTF0
  expanded.set(bytes.subarray(4, headerEnd), 4)
  const output = new DataView(expanded.buffer)
  let scaleIndex = 0
  let weightIndex = 0
  for (const tensorCount of tensorCounts) {
    for (
      let blockStart = 0;
      blockStart < tensorCount;
      blockStart += blockSize
    ) {
      const scale = source.getFloat32(scalesStart + scaleIndex * 4, true)
      const blockLength = Math.min(blockSize, tensorCount - blockStart)
      for (let i = 0; i < blockLength; i++) {
        const byte = bytes[weightsStart + weightIndex]
        const quantized = byte > 127 ? byte - 256 : byte
        output.setFloat32(
          8 + headerLength + weightIndex * 4,
          quantized * scale,
          true,
        )
        weightIndex++
      }
      scaleIndex++
    }
  }
  return expanded
}

export async function loadNtf(url: string) {
  const stored = new Uint8Array(await (await fetch(url)).arrayBuffer())
  const magic = ASCII.decode(stored.subarray(0, 4))
  const engine =
    magic === 'NTHF'
      ? expandF16(stored)
      : magic === 'NTQ8'
        ? expandInt8(stored)
        : stored
  return { engine, storedBytes: stored.length }
}

#!/usr/bin/env python3
"""Measure lower-precision encodings of a field-model .ntf file.

The compact files measure storage only. The round-trip files restore the
quantized values to f32 so the current engine can render them for comparison.
"""

import json
import struct
import sys
from pathlib import Path

import numpy as np

INT8_BLOCK_SIZE = 256


def header_and_weights(data):
    if data[:4] != b"NTF0":
        raise ValueError("not an NTF0 file")
    header_length = struct.unpack("<I", data[4:8])[0]
    header_bytes = data[8 : 8 + header_length]
    header = json.loads(header_bytes)
    weights = np.frombuffer(data, dtype="<f4", offset=8 + header_length)
    return header_length, header_bytes, header, weights


def tensor_ranges(header):
    offset = 0
    for tensor in header["tensors"]:
        count = int(np.prod(tensor["shape"]))
        yield tensor["name"], offset, offset + count
        offset += count


def write_roundtrip(path, header_bytes, weights):
    path.write_bytes(
        b"NTF0"
        + struct.pack("<I", len(header_bytes))
        + header_bytes
        + weights.astype("<f4").tobytes()
    )


def main():
    if len(sys.argv) != 3:
        raise SystemExit("usage: measure_model_compression.py <font.ntf> <output-dir>")

    source = Path(sys.argv[1])
    output_dir = Path(sys.argv[2])
    output_dir.mkdir(parents=True, exist_ok=True)
    data = source.read_bytes()
    _, header_bytes, header, weights = header_and_weights(data)

    f16 = weights.astype("<f2")
    f16_roundtrip = f16.astype("<f4")

    int8_chunks = []
    int8_roundtrip = np.empty_like(weights)
    scales = []
    for _, start, end in tensor_ranges(header):
        tensor = weights[start:end]
        peak = float(np.max(np.abs(tensor)))
        scale = peak / 127 if peak else 1.0
        quantized = np.clip(np.rint(tensor / scale), -127, 127).astype(np.int8)
        int8_chunks.append(quantized.tobytes())
        int8_roundtrip[start:end] = quantized.astype(np.float32) * scale
        scales.append(scale)

    block_chunks = []
    block_roundtrip = np.empty_like(weights)
    block_scales = []
    for _, start, end in tensor_ranges(header):
        for block_start in range(start, end, INT8_BLOCK_SIZE):
            block_end = min(block_start + INT8_BLOCK_SIZE, end)
            block = weights[block_start:block_end]
            peak = float(np.max(np.abs(block)))
            scale = peak / 127 if peak else 1.0
            quantized = np.clip(np.rint(block / scale), -127, 127).astype(np.int8)
            block_chunks.append(quantized.tobytes())
            block_roundtrip[block_start:block_end] = quantized.astype(np.float32) * scale
            block_scales.append(scale)

    (output_dir / "gulzar-f16.ntf").write_bytes(
        b"NTHF" + struct.pack("<I", len(header_bytes)) + header_bytes + f16.tobytes()
    )
    (output_dir / "gulzar-int8.compact").write_bytes(
        header_bytes + np.asarray(scales, dtype="<f4").tobytes() + b"".join(int8_chunks)
    )
    block_preamble = (
        b"NTQ8"
        + struct.pack("<I", len(header_bytes))
        + header_bytes
        + struct.pack("<II", INT8_BLOCK_SIZE, len(weights))
    )
    (output_dir / "gulzar-int8-block256.ntf").write_bytes(
        block_preamble
        + np.asarray(block_scales, dtype="<f4").tobytes()
        + b"".join(block_chunks)
    )
    write_roundtrip(output_dir / "gulzar-f16-roundtrip.ntf", header_bytes, f16_roundtrip)
    write_roundtrip(output_dir / "gulzar-int8-roundtrip.ntf", header_bytes, int8_roundtrip)
    write_roundtrip(
        output_dir / "gulzar-int8-block256-roundtrip.ntf", header_bytes, block_roundtrip
    )

    print(f"f32:  {len(data):,} bytes")
    print(f"f16:  {(output_dir / 'gulzar-f16.ntf').stat().st_size:,} bytes")
    print(f"int8: {(output_dir / 'gulzar-int8.compact').stat().st_size:,} bytes")
    print(
        "int8 block-256: "
        f"{(output_dir / 'gulzar-int8-block256.ntf').stat().st_size:,} bytes"
    )
    print(f"f16 maximum weight error: {np.max(np.abs(weights - f16_roundtrip)):.8f}")
    print(f"int8 maximum weight error: {np.max(np.abs(weights - int8_roundtrip)):.8f}")
    print(
        "int8 block-256 maximum weight error: "
        f"{np.max(np.abs(weights - block_roundtrip)):.8f}"
    )


if __name__ == "__main__":
    main()

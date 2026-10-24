---
title: "1.1 Memory-mapping GGUF tensor data in Zig"
date: 2026-10-24T09:30:00+01:00
draft: true
description: "Decoding the Q4_K and Q6_K quantized block layouts from ggml, then mmapping a GGUF file in Zig and pointing each tensor at its data."
---

This continues from [part 1]({{< ref "zig-2.md" >}}) of the [ZigLM series]({{< ref "zig-1.md" >}}).

So far we parsed the GGUF file and loaded it into memory, defining `Tensor` but leaving the `data` part out. Each `Tensor` we read contains an `offset` to a location in the data region of the GGUF file. This data region makes up the majority of the file, and is where mmap gains the most benefit as we don't need to store multiple copies of the tensor in memory.

## Tensor types in the file

Tensors in a GGUF file are `GGML` types which are defined in the [`ggml-common.h` file](https://github.com/ggml-org/ggml/blob/master/src/ggml-common.h). I will use this file to encode the structs we need to hold the tensor data. Printing the type of every tensor in the file shows there are only 3 kinds that we need to implement, for example:

```
Tensor: output.weight, shape: { 2048, 32000 }, type: GGML_TYPE_Q6_K
Tensor: token_embd.weight, shape: { 2048, 32000 }, type: GGML_TYPE_Q4_K
Tensor: blk.0.attn_norm.weight, shape: { 2048 }, type: GGML_TYPE_F32
```

The model is `tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf`. The `_M` means it is a "medium" mix: most weights are `Q4_K`, but some more sensitive tensors (like `output.weight`) are kept at the higher precision `Q6_K`, and small tensors like the norms stay as `F32`.

I am paraphrasing from [the intro to ggml](https://huggingface.co/blog/introduction-to-ggml) but some of the core points that are useful here:
- GGML is a machine learning library focused on Transformer inference, similar to PyTorch and TensorFlow.
- GGML is minimal, lightweight (binary < 1MB), and easy to compile.
- Support for quantized tensors.

The last point is important as some of the tensors in our GGUF file are quantized: `GGML_TYPE_Q6_K` and `GGML_TYPE_Q4_K`.

## Quantization and k-quants

A quick note on quantization (see [docs](https://ggml-org-ggml.mintlify.app/concepts/quantization)) and naming to help us understand what `Q6_K` and `Q4_K` mean:
- Quantization maps 32-bit floats to smaller integer representations.
- `Qn` means the weights are quantized to roughly n bits per weight.
- The `_K` suffix means the "k-quant" super-block format introduced in [llama.cpp PR #1684](https://github.com/ggml-org/llama.cpp/pull/1684), which uses multiple scales per block.

The idea behind quantization is to represent each of the model's weights in fewer bits (essentially mapping floating-point values to lower-precision integers), and reconstruct an approximation of the weight at inference time. The trade-off is reduced accuracy for reduced model memory footprint.

There are two kinds of quantization schemes:
- Symmetric: `w = d * q` - assumes the weights are symmetric around 0.
- Asymmetric: `w = d * q - m` - does not assume the weights are symmetric around 0.

Where `d` is a float scale, `q` is the stored integer value, and `m` is an offset to shift the whole range if it is not symmetric around 0 (ggml stores this "min" and subtracts it).

A tensor is an N-dimensional array of weights, and using a single float scale `d` for the whole matrix can lose precision. E.g. a naive way to quantize would be to find the largest value in the whole tensor and scale based on that value. However if that largest value is an outlier (e.g. 10x bigger than other weights) then precision is lost in other weights.

Instead, "blocking" is introduced, to compute a scale per block of weights, that way any outlier only affects a single block. Smaller blocks means more accuracy, but also more to store so there is more overhead per weight.

K-quants are an optimization to get small block accuracy without as much overhead, by introducing two block levels: "super-blocks" and "sub-blocks". Each super-block is divided into sub-blocks, and the sub-block scales and mins are themselves quantized, using a single float scale stored once per super-block.

Looking at the second tensor in the file `token_embd.weight` it has 2048 x 32000 = 65,536,000 weights. With `Q4_K` quantization, the tensor is broken into super-blocks of 256 elements (`QK_K`), so 65,536,000 / 256 = 256,000 super-blocks.

## The Q4_K super-block

Each of the tensor's super-blocks is represented as follows (taken from [ggml-common.h](https://github.com/ggml-org/ggml/blob/master/src/ggml-common.h) and converted to zig):

```zig
const QK_K = 256;
const K_SCALE_SIZE = 12;
const BlockQ4K = extern struct {
    d: f16, // super-block scale for quantized scales
    dmin: f16, // super-block scale for quantized mins
    scales: [K_SCALE_SIZE]u8, // scales and mins, quantized with 6 bits
    qs: [QK_K / 2]u8, // 4-bit quants
};
```

The `extern` keyword is required because zig structs do not guarantee a specific field order or memory layout, `extern` forces it to match the layout of a C struct exactly.

When working with C, zig also has [packed structs](https://zig.guide/working-with-c/packed-structs). These pack fields down to the bit with no padding, which is useful for things like C bit fields or flags.

They can't hold arrays like `qs`, so `extern` is what we need here.

Above we can see that the `Q4_K` super-block is asymmetric (contains mins) and `scales` is a 12 byte (96 bit) array holding the quantization scheme for the sub-blocks. Each sub-block needs a 6-bit scale and a 6-bit min, so we can see that there are 96/12 = 8 sub-blocks per super-block in this scheme. The super-block covers 256 weights / 8 sub-blocks = 32 weights per sub-block. Each of the weights is encoded as 4 bits in `qs`, requiring 128 bytes to hold all of them.

From the above, we can also derive the bits per weight for one `Q4_K` super-block as:

| Field | Bytes | Bits |
|---|---|---|
| `d` (f16) | 2 | 16 |
| `dmin` (f16) | 2 | 16 |
| `scales` | 12 | 96 |
| `qs` (256 × 4 bits) | 128 | 1024 |
| **total** | **144** | **1152** |

```
1152 bits / 256 weights = 4.5 bits per weight
```

Which is a way of comparing different quantization schemes, e.g.:

- `GGML_TYPE_Q4_K`: 4.5 bits/weight
- `GGML_TYPE_Q6_K`: 6.5625 bits/weight (210 bytes × 8 / 256 weights)

## The Q6_K super-block

The `Q6_K` super-block in zig:

```zig
const BlockQ6K = extern struct {
    ql: [QK_K / 2]u8, // quants, lower 4 bits
    qh: [QK_K / 4]u8, // quants, upper 2 bits
    scales: [QK_K / 16]i8, // scales, quantized with 8 bits
    d: f16, // super-block scale
};
```

This is a symmetric scheme (no mins) which covers 256 weights broken up into 16 sub-blocks, 256/16 = 16 weights per sub-block. Here the scales are quantized with 8 bits instead of 6 bits in `Q4_K`.

## Comparing sizes

It's also interesting to compare these tensors with the size they would be without quantization:

| Type | Calculation | Bytes | MiB |
|---|---|---|---|
| f32 | 65,536,000 × 4 | 262,144,000 | 250.0 |
| f16 | 65,536,000 × 2 | 131,072,000 | 125.0 |
| Q6_K | (65,536,000 / 256) × 210 | 53,760,000 | 51.3 |
| Q4_K | (65,536,000 / 256) × 144 | 36,864,000 | 35.2 |

## Type traits and tensor length

Now we have fully defined the tensor types in the GGUF file we can point each tensor's data at the correct place in the data region.

I trimmed `GGMLType` down to just the 3 types this model uses:

```zig
pub const GGMLType = enum(u32) {
    GGML_TYPE_F32 = 0,
    GGML_TYPE_Q4_K = 12,
    GGML_TYPE_Q6_K = 14,
};
```

Next, we need to define some traits for each of the GGML types so we know how many bytes each type takes up so we can calculate the total length of each tensor's data, following [ggml.c](https://github.com/ggml-org/ggml/blob/da8e2a54b919557b816b1c2beb432ccb766f7ce0/src/ggml.c#L643):

```zig
const TypeTraits = struct { name: []const u8, blck_size: u32, type_size: u32, is_quantized: bool };

pub fn traits(t: GGMLType) TypeTraits {
    return switch (t) {
        .GGML_TYPE_F32 => .{ .name = "f32", .blck_size = 1, .type_size = @sizeOf(f32), .is_quantized = false },
        .GGML_TYPE_Q4_K => .{ .name = "q4_K", .blck_size = QK_K, .type_size = @sizeOf(BlockQ4K), .is_quantized = true },
        .GGML_TYPE_Q6_K => .{ .name = "q6_K", .blck_size = QK_K, .type_size = @sizeOf(BlockQ6K), .is_quantized = true },
    };
}
```

The `Tensor` struct gets a new `data` field, which `Tensor.parse` initialises to an empty slice until we know where the data region starts:

```zig
data: []const u8,
...
return Tensor{
    .name = name,
    .n_dimensions = n_dimensions,
    .dimensions = dimensions,
    .type = tensor_type,
    .offset = offset,
    .data = &.{},
};
```

To fill it in we need the length of each tensor in bytes, which can be calculated as follows:

```
length = num blocks x bytes per block
where num blocks = num elements / elements per block
elements per block is the number of weights one block encodes (e.g. 1 for F32, 256 for Q4_K, Q6_K)

number of elements is e.g. 2048*32000 for the shape: {2048, 32000} output.weight tensor shown at the start.
```

Encoded as a function on `Tensor`:

```zig
pub fn byte_len(self: *Tensor) usize {
    var elements: u64 = 1;
    for (self.dimensions) |d| elements *= d;

    const trait = ggml.traits(self.type);
    const n_blocks = elements / trait.blck_size;
    return n_blocks * trait.type_size;
}
```

## Memory-mapping the file

To mmap the gguf file in zig is straightforward:

```zig
const len: usize = @intCast(try file.length(io));
var mm = try file.createMemoryMap(io, .{
    .len = len,
    .protection = .{ .read = true }, // read only
    .populate = false,
});
defer mm.destroy(io);
```

`mm.memory` is now a `[]const u8` covering the whole file. Because of the `defer`, the mapping only lives until the end of the function, and every tensor's `data` slice points into it. As with `deinit` in part 1, once we use the tensors outside `load`, the `mm.destroy` will need to move up to the caller.

## Finding the data region

In [part 1]({{< ref "zig-2.md" >}}) the parser read the file through a streaming reader with a 4 KiB buffer. Now that the whole file is mapped, we can switch to a fixed reader over the mapped bytes, so that the reader's `seek` position is the position inside the file:

```zig
fn initFromMmap(allocator: std.mem.Allocator, memory: []const u8) !Gguf {
    var r = std.Io.Reader.fixed(memory);
    // ... parse the header, metadata and tensor info
```

After parsing the metadata and tensor info, `r.seek` is the number of bytes consumed from the start of the file. That nearly gets us to the start of the tensor data. However, we also need to consider the alignment of the gguf file, because the end of the tensor info section might not align with the start of the tensor data.

So we need to read the alignment of the file from the metadata, which [defaults to 32](https://github.com/ggml-org/ggml/blob/master/docs/gguf.md) if it isn't set:

```zig
const alignment: u32 = if (metadata.get("general.alignment")) |kv|
    kv.value.GGUF_METADATA_VALUE_TYPE_UINT32
else
    32;
```

Then align forward to the next multiple of the alignment:

```zig
const data_start = std.mem.alignForward(usize, r.seek, alignment);
```

## Slicing the tensors

Each tensor's `offset` is relative to the start of the data region, so its data starts at `data_start + offset` in the mapped memory:

```zig
for (tensors.items) |*tensor| {
    const start = data_start + tensor.offset;
    const len = tensor.byte_len();
    if (start + len > memory.len) return error.TensorOutOfBounds;
    tensor.data = memory[start..][0..len];
}
```

Every tensor's `data` now points directly at its bytes in the mapped file, without copying anything.

---
title: "ZigLM: Learning zig and local inference"
date: 2026-05-29T16:57:04+01:00
draft: false
---

I recently came across [this video](https://www.youtube.com/watch?v=iqddnwKF8HQ) from the Zig creator which inspired me to check out the language. I also have been playing around with running local LLM models using llamacpp and wanted to learn a bit more deeply about how that works.

I'm trying to use Claude to learn here rather than generate everything, so I can dig more deeply into the code. I think that Claude can be a very useful tool for learning quickly if used responsibly, but it requires a bit of effort to get the balance right to ensure you are still learning.

Claude generated a plan that I am going to use for a series of blog posts as I document my journey.

---

# A learning path from zero to GPU inference
## Step 1: GGUF parser (CPU only, pure Zig)
Parse a GGUF file and print all tensor names, shapes, and quantization types. No math yet. Teaches you Zig file I/O, mmap, packed structs, and the format itself. This is ~200 lines.
## Step 2: CPU-only matrix multiplication
Implement a naive matmul(A, B) -> C for f32 tensors. Then add SIMD optimisation using Zig's @Vector builtins. You'll feel why GPU offload matters once you time it.
## Step 3: Tokenizer
Parse the BPE vocab from the GGUF and implement encode/decode. This teaches you about Zig's string handling and hash maps.
## Step 4: Tiny transformer, CPU only
Implement a single Llama-architecture forward pass in pure Zig using f32 weights. Run a small model (Llama 3.2 1B or SmolLM 135M). This is the big learning leap — forces you to understand every operation.
## Step 5: Add GPU compute
Pick one of:

Vulkan compute shaders — cross-platform (Windows/Linux/AMD/NVIDIA). Write GLSL/SPIR-V kernels for matmul and dequantization. Zig can call Vulkan via @cImport on the C headers.
Metal (macOS/Apple Silicon) — write MSL kernels, call Metal API from Zig. zinc and zllm both do this and are readable examples.
CUDA — NVIDIA only. You write .cu kernel files, compile them separately with nvcc, and call the resulting .so from Zig. Zig's build system can invoke nvcc as a custom build step.
WebGPU (wgpu) — most portable long-term, can target browser too. There are early Zig bindings.

The GPU work is essentially: allocate a buffer, upload tensors, dispatch a compute shader that does matmul/dequantize, read back the result.

---

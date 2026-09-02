---
title: "Zig 1"
date: 2026-05-29T16:57:04+01:00
draft: true
---

I have been feeling quite down recently about losing one of by hobbies, programming. If I had a few spare hours on a weekend/evening and felt the itch I would sit down and tinker on a computer, learning a new language or working on a side project. More recently my side projects have become prompts via claude code, generating code quickly. I can generate an idea fast, but the joy of the craft has gone. My desire to continue learning new programming languages has not disappeared,  but felt a bit pointless, like a waste of time. 

However, I recently came across this video about Zig which really inspired me, because it made me realize that I can still do my hobby if I enjoy it. And I thought it would be fun to learn Zig and also use this opportunity to learn more about running LLM models locally. I have used llamacpp and ollama to run Gemma 4 with Opencode. I enjoyed watching the logging from ollama as the inference was happening, longing to understand more deeply what was happening under the hood.

So I fired up claude code, this time not to begin coding the side project for me, but to layout a side project idea for me:

"I want to learn Zig and I want to learn about how to run openweight LLM models locally, e..g like llamaCPP, could i write something like that in zig that uses gpu/cpu code?"

And here's what I got back:

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

This time I will not be using Claude to generate any code for me for this project. I am going to follow the plan, but I'm going to write the code myself and enjoy it because why not? When I was younger, I built a crystal radio with my Granddad, we had radios, tvs etc. but it was still fun.

So I'm going to work through a series of blog posts, implementing these steps in Zig, and document my journey, just like the good old days.

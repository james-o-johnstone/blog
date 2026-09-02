---
title: "Zig 2."
date: 2026-05-29T17:08:18+01:00
draft: true
---

# 1. Implementing a GGUF parser in Zig

[GGUF = GPT-Generated Unified Format](https://github.com/ggml-org/ggml/blob/master/docs/gguf.md)

Following along with the [zig-book](https://pedropark99.github.io/zig-book/Chapters/01-zig-weird.html), I started by running `zig init` and then creating a new `gguf.zig` file. o

In this file I added a new function:

`pub fn parse(filename: )`

And here I reached my first hurdle, how to declare strings in Zig. Well turns out there is no built in string type, so we need to use primitives like C. A string is an array of bytes `[] u8`.

`pub fn parse(filename: [] u8)`

And let's return nothing and try to print it:

```
const std = @import("std");

pub fn parse(filename: []u8) !void {
    std.debug.print(filename);
}

test "parse" {
    try parse("gguf.zig");
}
```

My first compiler error:
```
src/gguf.zig:8:15: error: expected type '[]u8', found '*const [8:0]u8'
    try parse("gguf.zig");
              ^~~~~~~~~~
src/gguf.zig:8:15: note: cast discards const qualifier
src/gguf.zig:3:24: note: parameter type declared here
pub fn parse(filename: []u8) !void {
```

After some time, reading the docs and Googling, I got to this point:

```
const std = @import("std");

pub fn parse(filename: []const u8) !void {
    std.debug.print("{s}\n", .{filename});
}

test "parse" {
    try parse("gguf.zig");
}
```

With a passing test:
```
[2026-05-29T16:45:48.194Z] Running test: gguf.zig - parse
1/1 gguf.test.parse...gguf.zig
OK
All 1 tests passed.
```

Which all took an upsettingly long time, and I know full well by this point an LLM could have generated all steps of the side project and I would probably hvae something up and running, but then I wouldn't have learned anything. So we will continue.

Next step: let's figure out how to load from a file. I got confused at this point, as most stuff online includes examples using [`std.fs`](https://ziglang.org/documentation/0.16.0/std/#std.fs) but it looks like in zig 0.16.0 which was release Apr 2026, all these [File System APIs were migrated to Io](https://ziglang.org/download/0.16.0/release-notes.html#File-System). So instead of `std.fs.openFile`, I need to use  `std.Io.Dir.openFile`.

At this point, I also need to understand how io works, as the `std.Io.Dir.openFile` requires `io` to be passed as an arg. Checking [the PR](https://github.com/ziglang/zig/pull/25592) where io was first introduced in Zig it looks like io shuould be initialized once in `main` and used throughout the code. The example in the PR used [`std.Io.Threaded`](https://ziglang.org/documentation/master/std/#std.Io.Threaded), there is a comment there in the docs that says the application is responsible for choosing the Io implementation and library code should accept an Io parameter rather than accessing directly. There is also an example in the `main.zig` that was generated initially that already shows how to use it in `main`:

```
pub fn main(init: std.process.Init) !void {
  const io = init.io;
}
```

so I can pass it into my load function:

```
pub fn load(io: std.Io, filename: []const u8) !void {
    const file = std.Io.Dir.cwd().openFile(io, filename, .{}) catch {
        std.debug.print("Failed to open file: {s}\n", .{filename});
        return;
    };
    std.debug.print("Successfully opened file: {s}\n", .{filename});
    defer file.close(io);
}
```

and call it:

```
pub fn main(init: std.process.Init) !void {
    const io = init.io;
    try gguf.load(io, "test.txt");
}
```

Now to read text from the file into a variable. For this I found [`zigbyexample`](https://zigbyexample.neocities.org/text-io) useful as a starting point as it gives an example of how to read from a file and print. However, we want to store the result in a variable. As we are eventually going to want to parse a gguf file, it will be useful to create a datatype to store the data in. Here I create a basic struct and extend the load method:

```
const Gguf = struct {
    field: []const u8,

    fn init(field: []const u8) Gguf {
        return Gguf{ .field = field };
    }

    fn print(self: Gguf) void {
        std.debug.print("Gguf field: {s}\n", .{self.field});
    }
};

pub fn load(io: std.Io, filename: []const u8) !void {
    var file = std.Io.Dir.cwd().openFile(io, filename, .{}) catch {
        std.debug.print("Failed to open file: {s}\n", .{filename});
        return;
    };
    var buf: [4096]u8 = undefined;
    var gguf: Gguf = undefined;
    var reader = file.readerStreaming(io, &buf);
    var r = &reader.interface;
    while (try r.takeDelimiter('\n')) |line| {
        gguf.field = line;
    }
    gguf.print();
    defer file.close(io);
}
```

Now all the basic components are working, next step is to start writing the parser. Looking at the structure of a GGUF file [here](https://github.com/ggml-org/ggml/blob/master/docs/gguf.md#file-structure), the basic structure is:

```
const Gguf = struct {
    magic: [4]u8,
    version: u4,
    tensor_count: u64,
    metadata_kv_count: u64,
    metadata: std.StringHashMap([]const u8),
    tensors: []Tensor,
};

const Tensor = struct { name: []const u8, n_dimensions: u32, dimensions: u64, type: u32, offset: u64 };
```

To test it out, I've also downloaded an example gguf file for a simple model [TinyLlama-1.1B-Chat-v1.0](https://huggingface.co/TinyLlama/TinyLlama-1.1B-Chat-v1.0) using `wget https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF/resolve/main/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf`. Catting the file, I can see some of the metadata fields:

```
GGUFÉgeneral.architecturellamageneral.name"tinyllama_tinyllama-1.1b-chat-v1.0llama.context_lengthllama.embedding_lengthllama.block_countllama.feed_forward_lengthllama.rope.dimension_count@llama.attention.head_count llama.attention.head_count_kv&llama.attention.layer_norm_rms_epsilon¬Å'7llama.rope.freq_base@Fgeneral.file_typetokenizer.ggml.modelllamatokenizer.ggml.tokens	}
```

I'll also need to switch the current `load` method to read bytes instead of text, using the binary-io example from [`zigbyexample`](https://zigbyexample.neocities.org/binary-io) to help me. I also discovered the [`std.mem.bytesAsValue`](https://ziglang.org/documentation/master/std/#std.mem.bytesAsValue) function which can be used to convert bytes to any type. This returns a pointer to a value of the specified type backed by those bytes, so I also need to dereference the pointer when initializing the struct. [Pointer deferencing](https://zig.guide/language-basics/pointers) in Zig is done using `variable.*` 

 I'm going to comment out some parts of the struct to start with, so we can just test out parsing the `magic` and `version` fields:

```
const Gguf = struct {
    magic: [4]u8,
    version: u4,

    fn initFromFile(io: std.Io, file: std.Io.File) !Gguf {
        var buf: [4096]u8 = undefined;
        var reader = file.readerStreaming(io, &buf);
        var r = &reader.interface;

        var mv_bytes: [8]u8 = undefined;
        _ = try r.readSliceShort(&mv_bytes);

        const magic = magic_version_bytes[0..4];
        const version = std.mem.bytesAsValue(u4, mv_bytes[4..8]);

        return Gguf{ .magic = magic.*, .version = version.* }; // deref using variable.*
    }

    fn print(self: Gguf) void {
        std.debug.print("Magic: {s}\n", .{self.magic});
        std.debug.print("Version: {x}\n", .{self.version});
    }
}


```

The load function then becomes:
```
pub fn load(io: std.Io, allocator: std.mem.Allocator, filename: []const u8) !void {
    var file = std.Io.Dir.cwd().openFile(io, filename, .{}) catch {
        std.debug.print("Failed to open file: {s}\n", .{filename});
        return;
    };
    var gguf: Gguf = try Gguf.initFromFile(io, allocator, file);
    gguf.print();
    defer file.close(io);
}
```

And turns out the `parse` function I added earlier wasn't really needed because I am handling the parsing in the struct.

Looks like this works! And gives the following:

```
% zig build run
Magic: GGUF
Version: 3
```

Let's also add some error handling and a test for this, files that don't start with GGUF or are not v3 are not valid.



Now I will continue parsing the rest of the fields.


Next step - convert to mmap

- note: we should mmap see https://tlbflush.org/post/2025_02_17_gguf_weekend/


other posts:
thoughts on zig vs rust
- much easier to get started and language seems simpler
- mcuh less available on internet, only std library docs not many examples

---
title: "1. Implementing a GGUF Parser in Zig"
date: 2026-10-05T17:08:18+01:00
draft: false
---

[GGUF = GPT-Generated Unified Format](https://github.com/ggml-org/ggml/blob/master/docs/gguf.md)

Repo: https://github.com/james-o-johnstone/ziglm

GGUF is a file format for storing models for inference with [GGML](https://huggingface.co/blog/introduction-to-ggml) which is a C/C++ ML library. The file contains the complete model definition including weights, tokenizer and architecture. We will start with loading this file as it is the first step to using a model for local inference. It's also a good first step for learning Zig fundamentals like types, IO, memory etc.

The layout of the file is here: [https://github.com/ggml-org/ggml/blob/master/docs/gguf.md#file-structure](https://github.com/ggml-org/ggml/blob/master/docs/gguf.md#file-structure)

Following along with the [zig-book](https://pedropark99.github.io/zig-book/Chapters/01-zig-weird.html), I started by running `zig init` and then creating a new `gguf.zig` file.

In this file I added a new function:

`pub fn parse(filename: )`

And here I reached my first hurdle, how to declare strings in Zig. Well turns out there is no built in string type, so we need to use primitives like C. In Zig, `[]u8` represents a slice of bytes, which can represent a string.

`pub fn parse(filename: []u8)`

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

Note the `!` before the return type here (`!void`) and also the `try`. There is some info [here](https://pedropark99.github.io/zig-book/Chapters/09-error-handling.html#returning-errors-from-functions) on error handling. `!` indicates the function might return an error, and `try` is a shortcut for `x catch |err| return err` in case you just want to bubble up the error, which is fine for now.

My first compiler error:

```
src/gguf.zig:8:15: error: expected type '[]u8', found '*const [8:0]u8'
    try parse("gguf.zig");
              ^~~~~~~~~~
src/gguf.zig:8:15: note: cast discards const qualifier
src/gguf.zig:3:24: note: parameter type declared here
pub fn parse(filename: []u8) !void {
```

The problem here: `[]u8` represents a slice of bytes with mutable elements and the string literal "gguf.zig" is a `[]const u8` i.e. it is immutable.

```
const std = @import("std");

pub fn parse(filename: []const u8) !void {
    std.debug.print("{s}\n", .{filename});
}

test "parse" {
    try parse("gguf.zig");
}
```

Next step: let's figure out how to load from a file. I got confused at this point, as most stuff online includes examples using `std.fs` but I found out that in zig 0.16.0 which was released Apr 2026, all these [File System APIs were migrated to Io](https://ziglang.org/download/0.16.0/release-notes.html#File-System). So instead of `std.fs.openFile`, I need to use `std.Io.Dir.openFile`.

Now I also need to understand how `io` works, as the `std.Io.Dir.openFile` requires `io` to be passed as an arg. Checking [the PR](https://github.com/ziglang/zig/pull/25592) where io was first introduced in Zig it looks like io should be initialized once in `main` and used throughout the code. 

The example in the PR used `std.Io.Threaded`, a comment in the docs says the application is responsible for choosing the Io implementation and library code should accept an Io parameter rather than accessing directly. 

I can pass it into my load function:

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

Now to read text from the file into a variable. For this I found [zigbyexample](https://zigbyexample.neocities.org/text-io) useful as a starting point, it gives an example of how to read from a file and print. As we are eventually going to want to parse a gguf file, let's create a datatype to store the data in. Here I create a basic struct and extend the load method:

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
    defer file.close(io);
    var buf: [4096]u8 = undefined;
    var gguf: Gguf = undefined;
    var reader = file.readerStreaming(io, &buf);
    var r = &reader.interface;
    while (try r.takeDelimiter('\n')) |line| {
        gguf.field = line;
    }
    gguf.print();
}
```

Now all the basic components are working, next step is to start writing the parser. Looking at the structure of a GGUF file [here](https://github.com/ggml-org/ggml/blob/master/docs/gguf.md#file-structure), the basic layout is:

```
const Gguf = struct {
    magic: [4]u8,
    version: u32,
    tensor_count: u64,
    metadata_kv_count: u64,
    metadata: std.StringHashMap([]const u8),
    tensors: std.ArrayList(Tensor),
};

const Tensor = struct { name: []const u8, n_dimensions: u32, dimensions: []const u64, type: u32, offset: u64 };
```

To test it out, I've also downloaded an example gguf file for a simple model [TinyLlama-1.1B-Chat-v1.0](https://huggingface.co/TinyLlama/TinyLlama-1.1B-Chat-v1.0) using `wget https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF/resolve/main/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf`. Catting the file, I can see some of the metadata fields:

```
GGUFÉgeneral.architecturellamageneral.name"tinyllama_tinyllama-1.1b-chat-v1.0llama.context_lengthllama.embedding_lengthllama.block_countllama.feed_forward_lengthllama.rope.dimension_count@llama.attention.head_count llama.attention.head_count_kv&llama.attention.layer_norm_rms_epsilon¬Å'7llama.rope.freq_base@Fgeneral.file_typetokenizer.ggml.modelllamatokenizer.ggml.tokens	}
```

I'll also need to switch the current `load` method to read bytes instead of text, using the [binary-io](https://zigbyexample.neocities.org/binary-io) example to help me. The [std.Io.reader](https://ziglang.org/documentation/0.16.0/std/#std.Io.Reader) interface also contains lots of functions for reading bytes from the stream into various types, e.g. `takeArray`, [`takeInt`](https://ziglang.org/documentation/master/std/#std.Io.Reader.takeInt). 

`takeArray` returns a `*[n]u8` i.e. it's a pointer to an array which points to a specific location in the 4096 byte buffer. On the next line, I dereference the pointer and copy the value with `const magic = m.*`. This is because we are reusing the same buffer to read the whole file, eventually the 4096 buffer will contain an entirely new set of bytes and data at that address would no longer be the same.

```
const Gguf = struct {
    const MAGIC = "GGUF";
    const VERSION = 3;

    magic: [4]u8,
    version: u32,

    fn initFromFile(io: std.Io, file: std.Io.File) !Gguf {
        var buf: [4096]u8 = undefined;
        var reader = file.readerStreaming(io, &buf);
        var r = &reader.interface;

        const m = try r.takeArray(4);
        const magic = m.*; // deref using .*
        const version = try r.takeInt(u32, .little);

        if (!std.mem.eql(u8, &magic, Gguf.MAGIC)) return error.InvalidFile;
        if (version != Gguf.VERSION) return error.UnsupportedVersion;

        return Gguf{ .magic = magic, .version = version }; 
    }

    fn print(self: Gguf) void {
        std.debug.print("Magic: {s}\n", .{self.magic});
        std.debug.print("Version: {x}\n", .{self.version});
    }
};
```

The load function then becomes the following, notice I have used the `catch` instead of `try` here to handle the error in place and print a debug line:

```
pub fn load(io: std.Io, filename: []const u8) !void {
    var file = std.Io.Dir.cwd().openFile(io, filename, .{}) catch {
        std.debug.print("Failed to open file: {s}\n", .{filename});
        return;
    };
    defer file.close(io);
    var gguf: Gguf = try Gguf.initFromFile(io, file);
    gguf.print();
}
```

```
% zig build run
Magic: GGUF
Version: 3
```

Now I will continue parsing the remaining fields. Fleshing out the rest of the GGUF struct requires introducing a few more types as defined in the [spec](https://github.com/ggml-org/ggml/blob/master/docs/gguf.md).
A lot of these structs are mechanical so I'm not going to repeat everything here but will document some interesting bits:

- `Tensors` make up the meat of the GGUF. Each `Tensor` contains an `offset` to a position in the tensor data region of the file where that tensor's data is stored.
- `GGMLType` is the type of the tensor, just looking at this shows that a GGML tensor can be one of [many types](https://github.com/james-o-johnstone/ziglm/blob/e865f6a522d2a4bc83c0414d7d6a849ef2fe163c/src/gguf.zig#L15).
- `GgufMetadataKvT` - the model metadata contains string keys mapped to many types from simple strings e.g. "general.architecture": "llama", [to arrays](https://github.com/james-o-johnstone/ziglm/blob/e865f6a522d2a4bc83c0414d7d6a849ef2fe163c/src/gguf.zig#L129) (which can themselves also contain arrays).

```
const Gguf = struct {
    const MAGIC = "GGUF";
    const VERSION = 3;

    magic: [4]u8,
    version: u32,
    tensor_count: u64,
    metadata_kv_count: u64,
    metadata: std.StringHashMap(GgufMetadataKvT),
    tensors: std.ArrayList(Tensor),
};

const Tensor = struct {
    name: GgufString,
    n_dimensions: u32,
    dimensions: []const u64,
    type: GGMLType,
    offset: u64,
};

const GGMLType = enum(u32) {
    GGML_TYPE_F32 = 0,
    GGML_TYPE_F16 = 1,
    GGML_TYPE_Q4_0 = 2,
    GGML_TYPE_Q4_1 = 3,
    // omitted rest of fields in this post but the rest can be seen in the repo
};

const GgufMetadataKvT = struct {
    key: GgufString,
    value_type: GgufMetadataValueType,
    value: GgufMetadataValue,
};

const GgufMetadataValueType = enum(u32) {
    GGUF_METADATA_VALUE_TYPE_UINT8 = 0,
    GGUF_METADATA_VALUE_TYPE_INT8 = 1,
    GGUF_METADATA_VALUE_TYPE_UINT16 = 2,
    GGUF_METADATA_VALUE_TYPE_INT16 = 3,
    // more fields in the repo
};

// zig tagged union
const GgufMetadataValue = union(GgufMetadataValueType) {
    GGUF_METADATA_VALUE_TYPE_UINT8: u8,
    GGUF_METADATA_VALUE_TYPE_INT8: i8,
    GGUF_METADATA_VALUE_TYPE_UINT16: u16,
    GGUF_METADATA_VALUE_TYPE_INT16: i16,
};
```

The `GgufMetadataValue` can be one of many `GgufMetadataValueType`, which can be expressed in Zig using a [tagged union](https://zig.guide/language-basics/unions/). The tag in this case is the `GgufMetadataValueType` enum which indicates which of the fields within the `GgufMetadataValue` union is active. The full implementation requires a switch to handle each of those types being active within the union:

```
const GgufMetadataValue = union(GgufMetadataValueType) {
    GGUF_METADATA_VALUE_TYPE_UINT8: u8,
    GGUF_METADATA_VALUE_TYPE_INT8: i8,
    GGUF_METADATA_VALUE_TYPE_UINT16: u16,
    GGUF_METADATA_VALUE_TYPE_INT16: i16,

    fn parse(reader: *std.Io.Reader, value_type: GgufMetadataValueType) !GgufMetadataValue {
        switch (value_type) {
            .GGUF_METADATA_VALUE_TYPE_UINT8 => {
                const value = try reader.takeInt(u8, .little);
                return GgufMetadataValue{ .GGUF_METADATA_VALUE_TYPE_UINT8 = value };
            },
            .GGUF_METADATA_VALUE_TYPE_INT8 => {
                const value = try reader.takeInt(i8, .little);
                return GgufMetadataValue{ .GGUF_METADATA_VALUE_TYPE_INT8 = value };
            },
            // etc
        }
    }
};
```

`GgufMetadataValueType` can also be an array, so now is also a good time to mention allocators in Zig.

There are a few different allocators to choose from (including the c allocator: `std.heap.c_allocator`), and it is also possible to implement a custom allocator. 

From the docs on [choosing an allocator](https://ziglang.org/documentation/0.16.0/#Choosing-an-Allocator) I decided to start with the general purpose allocator (gpa), which in debug builds is a `DebugAllocator`, the "safe allocator" (the switch can be seen in `std/start.zig` [here](https://codeberg.org/ziglang/zig/src/branch/master/lib/std/start.zig#L781)). This is a good starting point for development as it can detect memory issues by tracking every allocation in the program.

The [std.mem.Allocator](https://ziglang.org/documentation/master/std/#std.mem.Allocator) interface provides

`pub fn alloc(self: Allocator, comptime T: type, n: usize) Error![]T`: Allocates an array of `n` items of type `T` and sets all the items to `undefined`

and 

`pub fn free(self: Allocator, memory: anytype) void` : Free an array allocated with `alloc`.

The guidance in the docs is to create the gpa once in the `main` function and then pass it around:

```
pub fn main(init: std.process.Init) !void {
    const io = init.io;
    const allocator = init.gpa;
    try gguf.load(io, allocator, "tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf");
}
```

I created a `GgufArray` struct and added calls to `alloc` and `free`  to show how they are used:

```
const GgufArray = struct {
    type: GgufMetadataValueType,
    len: u64,
    array: []const GgufMetadataValue,

    fn parse(reader: *std.Io.Reader, allocator: std.mem.Allocator) !GgufArray {
        const value_type = try reader.takeEnum(GgufMetadataValueType, .little);
        const len = try reader.takeInt(u64, .little);

        const arr = try allocator.alloc(GgufMetadataValue, len);
        var parsed: usize = 0;
        errdefer {
            for (arr[0..parsed]) |value| value.deinit(allocator);
            allocator.free(arr);
        }

        for (arr) |*value| {
            value.* = try GgufMetadataValue.parse(reader, value_type, allocator);
            parsed += 1;
        }

        return GgufArray{ .type = value_type, .len = len, .array = arr };
    }

    pub fn deinit(self: *const GgufArray, allocator: std.mem.Allocator) void {
        for (self.array) |value| {
            value.deinit(allocator);
        }
        allocator.free(self.array);
    }
};
```

Allocators are also used for things like `std.ArrayList`, e.g. for the tensors:

```
var tensors = std.ArrayList(Tensor).empty;
errdefer {
    for (tensors.items) |*t| t.deinit(allocator);
    tensors.deinit(allocator);
}
for (0..tensor_count) |_| {
    var tensor = try Tensor.parse(r, allocator);
    errdefer tensor.deinit(allocator);
    try tensors.append(allocator, tensor);
}
```

There is no GC in Zig so nothing automatically frees memory when it goes out of scope, we need to remember to call `free` on everything that was allocated. Zig's `defer`/`errdefer` can handle this as it is guaranteed to run when the scope exits.

So on the `Gguf` struct I added a `deinit` method which walks everything that it created to ensure it is freed:

```
pub fn deinit(self: *Gguf, allocator: std.mem.Allocator) void {
    var it = self.metadata.iterator();
    while (it.next()) |entry| {
        entry.value_ptr.*.deinit(allocator);
    }
    self.metadata.deinit();
    for (self.tensors.items) |*item| {
        item.deinit(allocator);
    }
    self.tensors.deinit(allocator);
}
```

Then in the `load` method which parses the file, we just need to call `defer gguf.deinit(allocator)` and the resources will be freed. Note that currently we are only printing the gguf in that `load` function and the gguf lifetime is tied to that function. When we want to use the gguf elsewhere in the program, we will need to move the defer up to the caller.

```
pub fn load(io: std.Io, allocator: std.mem.Allocator, filename: []const u8) !void {
    var file = std.Io.Dir.cwd().openFile(io, filename, .{}) catch {
        std.debug.print("Failed to open file: {s}\n", .{filename});
        return;
    };
    defer file.close(io);
    var gguf = try Gguf.initFromFile(io, allocator, file);
    defer gguf.deinit(allocator);
    gguf.print();
}
```

Here the benefit of starting with the gpa allocator paid off. When I was implementing the `Tensor` parsing, I forgot to free the `dimensions` array, resulting in a runtime error:

```
error(DebugAllocator): memory address 0x109020300 leaked:
ziglm/src/gguf.zig:88:47: 0x100d0e21f in parse (ziglm)
        const dimensions = try allocator.alloc(u64, n_dimensions);
                                              ^
ziglm/src/gguf.zig:367:44: 0x100d0f4ab in initFromFile (ziglm)
            const tensor = try Tensor.parse(r, allocator);
                                           ^
ziglm/src/gguf.zig:409:37: 0x100d0f88b in load (ziglm)
    var gguf = try Gguf.initFromFile(io, allocator, file);
```

So a leak like this would not be detected with non-debug/testing allocators, leading to some silent issues at runtime.

The full gguf.zig file can be seen [here](https://github.com/james-o-johnstone/ziglm/blob/main/src/gguf.zig).

The next thing to do is mmap the tensor data. The file we are testing with is ~638MiB however the parts we have parsed so far make up only ~1.6MiB of that so the majority of the data has not been read yet. We need to mmap the tensor data so we get a pointer to the bytes in the page cache (the kernel's in memory cache of the file) rather than needing to make a second copy in RAM. 

The model we are using here is just a small 1.1B parameter model so it could comfortably fit multiple copies, but if we look at some [gemma-4-31B](https://huggingface.co/unsloth/gemma-4-31B-it-GGUF/tree/main) GGUFs then they can reach 30-40GiB.

One other benefit of mmap is "lazy paging" i.e. the OS never reads the parts of the file that aren't touched, and they can be loaded on demand as needed.

This mmap will be covered in the next post.

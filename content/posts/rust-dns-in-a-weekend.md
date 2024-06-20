---
title: "Implement DNS in a Weekend (in Rust!) Part 1"
date: 2024-06-19T15:34:14+01:00
summary: To improve my skills in Rust, learn more about how DNS works and eventually learn about Rust/Ruby and Rust/Java interop I have decided to follow along with "Implement dns in a weekend", reimplementing the Python code in Rust.
draft: false
---

# Introduction

To improve my skills in Rust, learn more about how DNS works and eventually learn about Rust/Ruby and Rust/Java interop I have decided to follow along with https://implement-dns.wizardzines.com/ which I saw on Hackernews at some point, and reimplement the Python code in Rust with a plan to eventually create a library that I can call from Ruby and Java. Full credit is given to https://implement-dns.wizardzines.com for the original "Implement dns in a weekend" content, I am just following along here using Rust instead of Python.

The full repo is here: https://github.com/james-o-johnstone/rust-dns-in-a-weekend

# 1.1 Write the `DNSHeader` and `DNSQuestion` classes

```rust
struct DNSHeader {
    id: u16, // query ID
    flags: u16, // some flags
    // 4 counts telling you how many records to expect in each section of a DNS packet:
    num_questions: u16,
    num_answers: u16,
    num_authorities: u16,
    num_additionals: u16
}
```

DNS question has 3 fields, name (like example.com), a type (like A), and a class (which is always the same):

```rust
struct DNSQuestion {
    name: String,
    r#type: u16, // escaped with r# to use as identifier
    class: u16
}
```

# 1.2 Convert these classes to bytes

In the original [book](https://implement-dns.wizardzines.com/book/part_1), there is some Python code to convert the above classes into byte strings. In rust we need to convert each class into a vector of bytes, represented as `Vec<u8>` (`u8` is a primitive 8-bit unsigned integer type which represents a byte).

Rather than introducing the `header_to_bytes` and `question_to_bytes` functions from the Python example, I will create a custom macro and generic trait so I can just annotate any structs I want to convert to bytes with `#derive[ToBytes]` and don't need to write a custom function for every struct. In rust these kinds of macros are called [Procedural Macros](https://doc.rust-lang.org/reference/procedural-macros.html).

I will need to create two new crates in the rust project. One for the procedural macro definition and the other to implement `to_bytes` for the primitive datatypes.

For the macro definition I will create a new crate with `cargo new struct_bytes_derive --lib` and edit the `Cargo.toml` to make it a procedural macro crate:

```
[lib]
proc-macro = true
```

And to implement `to_bytes` I will create another crate `cargo new struct_bytes` and implement support for `String` and `u16` datatypes, as these are the only datatypes making up the DNS structs so far. Note that as mentioned in the original book, we need to ensure that we encode integer bytes as big endian as they will be sent over the wire:

```rust
pub trait ToBytes {
    fn to_bytes(&self) -> Vec<u8>;
}

impl ToBytes for u16 {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec() // encode integers as big endian
    }
}

impl ToBytes for String {
    fn to_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}
```

In order to implement the macros in `struct_bytes_derive`, I used the `syn` and `quote` crates to help with parsing and writing the Rust code. To help get me started I followed the [heapsize example](https://github.com/dtolnay/syn/blob/master/examples) to handle parsing for me.

My derive macro ended up looking like this:

```rust
#[proc_macro_derive(ToBytes)]
pub fn derive_tobytes(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = input.ident;

    // Generate an expression to convert each field to bytes
    let bytes = to_bytes(&input.data);

    let expanded = quote! {
        // The generated impl.
        impl struct_bytes::ToBytes for #struct_name {
            fn to_bytes(&self) -> Vec<u8> {
                #bytes
            }
        }
    };

    // Hand the output tokens back to the compiler.
    TokenStream::from(expanded)
}

// Generate an expression to convert each field to bytes
fn to_bytes(data: &Data) -> proc_macro2::TokenStream {
    match *data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let recurse = fields.named.iter().map(|f| {
                        let name = &f.ident;
                        quote! {
                            &self.#name.to_bytes()
                        }
                    });
                    quote! {
                        let mut bytes = Vec::<u8>::new();
                        #(bytes.extend(#recurse);)* // repeats bytes.extend for every element of recurse to populate the vector
                        bytes
                    }
                }
                Fields::Unnamed(_) | Fields::Unit => unimplemented!()
            }
        }
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}
```

Now we can add the trait to our classes:

```rust
use struct_bytes::ToBytes;

#[derive(ToBytes)]
struct DNSHeader { ... }

#[derive(ToBytes)]
struct DNSQuestion { ... }
```

And the compiler will generate the implementations automagically as follows:

```
impl struct_bytes::ToBytes for DNSHeader
{
    fn to_bytes(&self) -> Vec<u8>
    {
        let mut bytes = Vec::<u8>::new();
        bytes.extend(&self.id.to_bytes());
        bytes.extend(&self.flags.to_bytes());
        bytes.extend(&self.num_questions.to_bytes());
        bytes.extend(&self.num_answers.to_bytes());
        bytes.extend(&self.num_authorities.to_bytes());
        bytes.extend(&self.num_additionals.to_bytes());
        bytes
    }
}
impl struct_bytes::ToBytes for DNSQuestion
{
    fn to_bytes(&self) -> Vec<u8>
    {
        let mut bytes = Vec::<u8>::new();
        bytes.extend(&self.name.to_bytes());
        bytes.extend(&self.r#type.to_bytes());
        bytes.extend(&self.class.to_bytes());
        bytes
    }
}
```

Let's validate by checking our header encodes to bytes in the same way as in the book:

```rust
let header = DNSHeader{ 
    id: 0x1314,
    flags: 0,
    num_questions: 1,
    num_answers: 0,
    num_authorities: 0,
    num_additionals: 0
};
println!("{:x?}", header.to_bytes());
```

```bash
$ cargo run
header: [13, 14, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0]
```

This matches the expected output from the Python example ✅

# 1.3 Encode the name

To build the DNS query we need to encode the domain name, so "google.com" is translated into `\x06google\x03com\x00`. The domain name is split into parts and each part is prepended with its length: `6 google 3 com 0.`

We will make this a part of the `DNSQuestion` struct implementation so that the class can be constructed with a domain name string which will be automatically converted to the name in the correct format:

```rust
impl DNSQuestion {
    fn encode_dns_name(domain_name: String) -> Vec<u8> {
        let mut encoded = Vec::<u8>::new();
        domain_name.split(".").for_each(|part| {
            encoded.push(part.len().try_into().unwrap()); // add the number of bytes of the part
            encoded.extend_from_slice(part.as_bytes());
        });
        encoded.push(b'\0'); // add zero byte
        encoded
    }
}
```

And now adding the constructor means we can create the `DNSQuestion` struct with a `domain_name`, without needing to first convert the domain name to the required format first, with e.g. `DNSQuestion::new("google.com", type, class)`:

```rust
impl DNSQuestion {
    pub fn new(domain_name: String, r#type: u16, class: u16) -> Self {
        Self {
            name: encode_dns_name(domain_name),
            r#type,
            class,
        }
    }
```

# 1.4 Build the query

In the Python book, the `build_query` function takes a domain name (e.g. `google.com`) and the number of a DNS record type (like A). The function in rust looks like this:

```rust
const TYPE_A: u16 = 1;
const CLASS_IN: u16 = 1;

fn build_query(domain_name: String, record_type: u16) -> Vec<u8> {
    let RECURSION_DESIRED = 1 << 8;
    let header = DNSHeader {
        id: rand::thread_rng().gen(),
        flags: RECURSION_DESIRED,
        num_questions: 1,
        num_answers: 0,
        num_authorities: 0,
        num_additionals: 0
    };
    let question = DNSQuestion::new(
        domain_name,
        record_type,
        CLASS_IN
    );
    let mut query = Vec::<u8>::new();
    query.append(header.toBytes().as_mut());
    query.append(question.toBytes().as_mut());
    query
}
```

# Test our code

Now we will create a main function which opens a UDP socket, builds a query to resolve "www.example.com" and sends it to Google's DNS resolver

```rust
use std::net::UdpSocket;
fn main() -> std::io::Result<()> {
    {
        let query = build_query(String::from("www.example.com"), TYPE_A);
        let socket = UdpSocket::bind("0.0.0.0:34254")?;
        socket.send_to(&query, "8.8.8.8:53")?;
        let mut buf = [0; 1024]; // UDP DNS responses are usually less than 512 bytes
        socket.recv_from(&mut buf)?;
    }
    Ok(())
}
```

To test the program, we start `tcpdump` and run the rust code with `cargo build && cargo run`

```bash
$ sudo tcpdump -ni any port 53
listening on any, link-type LINUX_SLL (Linux cooked v1), capture size 262144 bytes
21:20:25.499980 IP 192.168.1.90.34254 > 8.8.8.8.53: 33432+ A? www.example.com. (33)
21:20:25.512366 IP 8.8.8.8.53 > 192.168.1.90.34254: 33432 1/0/0 A 93.184.215.14 (49)
```

And we can see the answer from 8.8.8.8. Success! 🎉

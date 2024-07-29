---
title: "Implement DNS in a Weekend (in Rust!) Part 3"
date: 2024-07-27T09:04:34+01:00
draft: false
summary: Part 3 of "Implement DNS in a weekend" - implementing the resolver.
---

# Part 3: Implement our resolver

In this part we switch from sending queries to `8.8.8.8` to resolve the IP address, to resolve the IP address ourselves.

# 3.1: Don't ask for recursion

Previously, we were making DNS requests to a recursive DNS resolver server so we set the `RECURSION_DESIRED` flag on the `DNSHeader`. Recursive DNS resolvers can either resolve the IP address from their cache or make recursive queries to other authoritative DNS servers to resolve the IP. Authoritative DNS servers are the sources of truth (they hold the actual DNS records that we need).

Now we will be making a request to an authoritative DNS server instead, so we disable recursion by settings `flags = 0` on the header.

# 3.2 Write a `send_query` function

This function will ask a DNS server about a domain name and return a `DNSPacket`:

```rust
fn send_query(ip_address: String, domain_name: String, record_type: u16) -> std::io::Result<(DNSPacket)> {
    let query = build_query(domain_name, record_type, 0);
    let socket = UdpSocket::bind("0.0.0.0:34254")?;
    socket.send_to(&query, (ip_address, 53))?;
    let mut buf = [0; 1024];
    socket.recv_from(&mut buf)?;
    let mut br = ByteReader::from_bytes(&buf);
    let packet = DNSPacket::from_bytes(&mut br);
    Ok(packet)
}
```

And to test that it's working:

```rust
fn main() -> std::io::Result<()> {
    {
        let packet = send_query(
            String::from("8.8.8.8"),
            String::from("example.com"), 
            TYPE_A
        )?;
        println!("{:?}", packet.answers[0]);
    }
    Ok(())
}
```

Which prints:

```rust
DNSRecord { name: DNSName { name: "example.com" }, type: 1, class: 1, ttl: 83, data: DNSData { ip: 93.184.215.14 } }
```

# 3.3 Improving our parsing a little bit

Curently we are only supporting `A` records, here we will add support for `NS` records.

The way I have done this is a bit overkill, but I wanted to play around with generics and traits a bit more, so let me try and explain what is going on.

First of all, as is shown in the Python code, we need to parse the `DNSData` differently, depending on the record type:
- `A` records contain an IP address
- `NS` records contain a DNS name

To handle this in Rust, I have introduced a new `RecordType` trait, which contains an `ID` field and a default `parse_data` implementation which can be used to convert the `DNSData` bytes into a `String` for pretty printing.

```rust
trait RecordType {
    const ID: u16; // identify the record type, 1 == A, 2 == NS etc.
    fn parse_data(buf: &mut ByteReader) -> String {
        let data_len = buf.read_u16().unwrap();
        let data = buf.read_bytes(data_len.try_into().unwrap()).unwrap();
        String::from_utf8(data).unwrap()
    }
}
```

Now we will create a struct for each of the `RecordTypes` we want to support (currently `A` and `NS`), and implement the specific parsing behaviour for each of them:

```rust
struct TypeA;
impl RecordType for TypeA {
    const ID: u16 = 1;
    fn parse_data(buf: &mut ByteReader) -> String {
        let data_len = buf.read_u16().unwrap();
        let data = buf.read_bytes(data_len.try_into().unwrap()).unwrap();
        let ip = Ipv4Addr::new(data[0], data[1], data[2], data[3]);
        ip.to_string()
    }
}

struct TypeNS;
impl RecordType for TypeNS {
    const ID: u16 = 2;
    fn parse_data(buf: &mut ByteReader) -> String {
        DNSName::decode_name(buf)
    }
}
```

Next we create a generic `DNSData` struct which contains a type parameter scoped to the `RecordType` trait. This `DNSData` can hold the parsed data for any `RecordType`, and it parses the data using the `parse_data` function implemented on the `RecordType`.

```rust
struct DNSData<R: RecordType> {
    record_type: PhantomData<R>
    data: String
}
impl<R: RecordType> FromBytes for DNSData<R> {
    fn from_bytes(buf: &mut ByteReader) -> Self {
        let data = R::parse_data(buf);
        Self {
            record_type: PhantomData,
            data: data
        }
    }
}
```

One thing to mention here is usage of [`PhantomData`](https://doc.rust-lang.org/std/marker/struct.PhantomData.html). This is required because we have an unused type parameter in this struct. We don't really need to store the type parameter, as we don't actually need to use the type parameter again once the `DNSData` struct has been built, however the compiler requires that we track the type that the struct is "tied" to which can be done using `PhantomData`.

Now, because we don't know the `RecordType` until we have parsed the `type` from the `DNSRecord` in the response, we can't make the `DNSRecord` take a type parameter. Instead we can introduce an `enum`, and add data to each field that ties that specific record type to the `DNSData` required:

```rust
enum DNSRecordData {
    A(DNSData<TypeA>),
    NS(DNSData<TypeNS>)
}
```

And finally, this can be used in the `DNSRecord` as follows:

```rust
struct DNSRecord {
    name: DNSName,
    r#type: u16,
    class: u16,
    ttl: u32,
    data: DNSRecordData
}
impl FromBytes for DNSRecord {
    fn from_bytes(buf: &mut ByteReader) -> Self {
        let name = DNSName::from_bytes(buf);
        let r#type = u16::from_bytes(buf);
        let class = u16::from_bytes(buf);
        let ttl = u32::from_bytes(buf);
        let data = match r#type {
           TypeA::ID => DNSRecordData::A(DNSData::<TypeA>::from_bytes(buf)),
           TypeNS::ID => DNSRecordData::NS(DNSData::<TypeNS>::from_bytes(buf)),
           _ => panic!("Invalid type {}", r#type)
        };
        Self { 
            name,
            r#type,
            class,
            ttl,
            data
         }
    }
}
```

We now have a `DNSRecord` that can store data for any type without it needing to know how to parse the data for that specific type.

In order to pretty print the data, we can implement the `Display` type along with some pattern matching on the enum type to access the underlying data:

```rust
enum DNSRecordData {
    A(DNSData<TypeA>),
    NS(DNSData<TypeNS>)
}
impl DNSRecordData {
    fn data(&self) -> String {
        match self {
            Self::A(d) => d.data.clone(),
            Self::NS(d) => d.data.clone(),
        }
    }
}
impl Display for DNSRecordData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data())
    }
}
```

Now the data can be printed as follows:

```rust
let packet = DNSPacket::from_bytes(&mut br);
println!("{}", packet.answers[0].data);
```

As mentioned, I realise this is stupidly overcomplicated for this implementation, a much simpler approach would have been something like this:

```rust
const TYPE_A: u16 = 1;
const TYPE_NS: u16 = 2;
struct DNSRecord {
    name: DNSName,
    r#type: u16,
    class: u16,
    ttl: u32,
    data: String
}
impl FromBytes for DNSRecord {
    fn from_bytes(buf: &mut ByteReader) -> Self {
        let name = DNSName::from_bytes(buf);
        let r#type = u16::from_bytes(buf);
        let class = u16::from_bytes(buf);
        let ttl = u32::from_bytes(buf);
        let data = match r#type {
           TYPE_A => decode_name(buf),
           TYPE_NS => ip_to_string(buf)
           _ => panic!("Invalid type {}", r#type)
        };
        Self { 
            name,
            r#type,
            class,
            ttl,
            data
         }
    }
}
```

But then there would have been less rust learnings...

{{< rawhtml >}}
<iframe src="https://giphy.com/embed/11i77XHsbqUcZa" width="480" height="269" style="" frameBorder="0" class="giphy-embed" allowFullScreen></iframe><p><a href="https://giphy.com/gifs/i-regret-nothing-chicken-11i77XHsbqUcZa">via GIPHY</a></p>
{{< /rawhtml >}}

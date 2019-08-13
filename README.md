# TNEF [![Build Status](https://travis-ci.org/newpavlov/tnef.svg?branch=master)](https://travis-ci.org/newpavlov/tnef) [![dependency status](https://deps.rs/repo/github/newpavlov/tnef/status.svg)](https://deps.rs/repo/github/newpavlov/tnef)
A basic [TNEF] parser.

TNEF file contains a stream of records called "attributes". Using `TnefReader`
you can read attributes stored in the provided TNEF buffer. At the moment we do
not handle parsing of attribute data outside of attachement attributes.

If you just want to unpack attachments stored in TNEF, you can use a
convenience function `read_attachements`.

[TNEF]: https://en.wikipedia.org/wiki/Transport_Neutral_Encapsulation_Format

## Usage example
```rust
for attribute in tnef::TnefReader::new(tnef_data)? {
    let (id, data) = attribute?;
    println!("{:?} {}", id, data.len());
}
```

## Minimum Supported Rust Version (MSRV)
This crate requires Rust 1.32.0 or later.

## License

All crates licensed under either of

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
 * [MIT license](http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

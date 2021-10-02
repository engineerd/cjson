# Canonical JSON serialization for Rust

![Build and Test](https://github.com/engineerd/cjson/workflows/Build%20and%20Test/badge.svg)
[![docs.rs](https://docs.rs/cjson/badge.svg?version=0.1.1)](https://docs.rs/cjson/0.1.1/cjson/)
[![Crates.io](https://img.shields.io/crates/v/cjson.svg)](https://crates.io/crates/cjson)

This is an implementation for a canonical JSON serializer that tries to be
compliant with [the OLPC minimal specification for canonical JSON][olpc].
Additionally, the implementation also tries to be fully compatible with the [Go
canonical JSON implementation][docker/go/canonical] used across the Docker and
Notary ecosystem. If you find any inconsistencies with the result of the
serialization, please open an issue.

Example - reading a JSON file and printing its canonical representation:

```rust
let res: serde_json::Value =
    serde_json::from_reader(input).expect("cannot deserialize input file");

println!(
    "{}",
    cjson::to_string(&res).expect("cannot write canonical JSON")
);
```

> Note: this crate aims to always be compilable to the `wasm32-unknown-unknown`
> and `wasm32-wasi` targets.

### Building and contributing

This project welcomes contributions of any kind, particularly additional test
cases.

To build:

```
$ cargo build
$ cargo test
```

The `testdata` directory is structured in the following way:

- in the root of the directory are JSON files whose name is represented by the
  SHA256 digest of their canonical JSON representation, as calculated using the
  [`github.com/docker/go/canonical`][docker/go/canonical] package. The test case
  will use compare the SHA256 digest obtained after serializing using this
  implementation, to the file name, and they are expected to be equal.

To add a new test case, you can use the [`canonjson`][canonjson] binary, which
is a CLI wrapper over the Go canonical JSON implementation:

```
$ go get github.com/technosophos/canonjson
$ canonjson target-file.json | sha256sum
```

At this point, rename `target-file.json` to the `<computed-SHA256>.json`, the
move it in the root of the `testdata` directory.

- the `errors` sub-directory contains valid JSON files (if the files are not
  valid JSON files, the tests will fail), but which contain characters that are
  not permitted in canonical JSON - so trying to represent them in canonical
  JSON should produce an error.

Finally, the `scripts/integration.sh` script contains a very rudimentary test of
the CLI from `main.rs` - and compares the digest of obtained there with the
digest obtained from serializing with the Go implementation. Ideally, we would
add more implementations of canonical JSON to test against. Note that you also
need the `canonjson` binary used earlier to execute this script.

### Notes and acknowledgement

- this implementation was initially based on the canonical JSON serialization
  used in the [Rust TUF crate][rust-tuf].

[olpc]: http://wiki.laptop.org/go/Canonical_JSON
[docker/go/canonical]: https://github.com/docker/go/tree/master/canonical/json
[canonjson]: https://github.com/technosophos/canonjson
[rust-tuf]: https://github.com/heartsucker/rust-tuf/

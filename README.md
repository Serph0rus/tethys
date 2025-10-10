# tethys
a somewhat unixy operating system.

## documentation
- [filesystem](FILESYSTEM.md)

## building
the following are required:
- rustup
- cargo
- a **nightly** x86_64-unknown-none cargo toolchain
- qemu-system-x86_64 (could also be listed in some package managers as part of qemu-full)
- llvm-tools-preview (**$ rustup component add llvm-tools-preview**)
otherwise, the project has been configured to run with simply **$ cargo build** or **$ cargo run**

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

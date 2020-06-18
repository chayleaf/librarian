[![crates.io](https://img.shields.io/crates/v/librarian.svg)](https://crates.io/crates/librarian)
[![docs.rs](https://docs.rs/librarian/badge.svg)](https://docs.rs/librarian/)

# Librarian - a Rust crate for downloading and linking to non-rust libraries from app build scripts

When I started using C libraries in Rust on Windows, what bothered me a lot was the fact that while normally building a crate just requires `cargo build`, if you have to link to a C library and especially if you have to use a DLL, the process becomes more complicated. To eliminate that, I made this crate for myself and am publishing it in hopem it will be useful to others.

Do note that right now the project is only tested in a limited amount of environments, it isn't stable - so I don't guarantee it will work for you, and I don't guarantee backwards compatibility. However if it doesn't work for you you're welcome to contribute!

It probably isn't feasible to integrate with all the different build systems there are to allow automatically fetching source and building it. While it makes me appreciate the Rust build system, it means automatic downloading will only work for prebuild libraries.

# (Current) Features

- Downloading .zip archives with C libraries (Will be extended to .tgz, could be extended to git/etc)
- A wrapper for adding a link search path (`cargo:rustc-link-search`)
- A way to copy .dll/.so files to the build folder (Currently only makes sense to do from app build scripts, not lib build scripts)
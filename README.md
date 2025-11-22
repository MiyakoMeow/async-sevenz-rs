[![Crate](https://img.shields.io/crates/v/async-sevenz.svg)](https://crates.io/crates/async-sevenz)
[![Documentation](https://docs.rs/async-sevenz/badge.svg)](https://docs.rs/async-sevenz)

This project is an async 7z compressor/decompressor written in pure Rust.

This is a fork of [sevenz-rust2](https://github.com/hasenbanck/sevenz-rust2), then translated the api to async with [async-compression](https://crates.io/crates/async-compression) by AI.

## Supported Codecs & filters

| Codec       | Decompression | Compression |
|-------------|---------------|-------------|
| COPY        | ✓             | ✓           |
| LZMA        | ✓             | ✓           |
| LZMA2       | ✓             | ✓           |
| BROTLI (*)  | ✓             | ✓           |
| BZIP2       | ✓             | ✓           |
| DEFLATE (*) | ✓             | ✓           |
| PPMD        | ✓             | ✓           |
| LZ4 (*)     | ✓             | ✓           |
| ZSTD (*)    | ✓             | ✓           |

(*) Require optional cargo feature.

| Filter        | Decompression | Compression |
|---------------|---------------|-------------|
| BCJ X86       | ✓             | ✓           |
| BCJ ARM       | ✓             | ✓           |
| BCJ ARM64     | ✓             | ✓           |
| BCJ ARM_THUMB | ✓             | ✓           |
| BCJ RISC_V    | ✓             | ✓           |
| BCJ PPC       | ✓             | ✓           |
| BCJ SPARC     | ✓             | ✓           |
| BCJ IA64      | ✓             | ✓           |
| BCJ2          | ✓             |             |
| DELTA         | ✓             | ✓           |

## WASM support

- [ ] WASM support is unable for now, please write issue or pull request if you have idea to fix the build problems.

### Original

WASM is supported, but you can't use the default features. We provide a "default_wasm" feature that contains
all default features with the needed changes to support WASM:

```bash
RUSTFLAGS='--cfg getrandom_backend="wasm_js"' cargo build --target wasm32-unknown-unknown --no-default-features --features=default_wasm
```

## Licence

Licensed under the [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0).

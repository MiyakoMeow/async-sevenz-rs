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

### Usage

```toml
[dependencies]
sevenz-rust2 = { version = "0.19" }
```

Decompress source file "data/sample.7z" to destination path "data/sample":

```rust
async_sevenz::decompress_file("data/sample.7z", "data/sample").expect("complete");
```

#### Decompress an encrypted 7z file

Use the helper function to encrypt and decompress source file "path/to/encrypted.7z" to destination path "data/sample":

```rust
async_sevenz::decompress_file_with_password("path/to/encrypted.7z", "data/sample", "password".into()).expect("complete");
```

## Compression

Use the helper function to create a 7z file with source path:

```rust
async_sevenz::compress_to_path("examples/data/sample", "examples/data/sample.7z").expect("compress ok");
```

### Compress with AES encryption

Use the helper function to create a 7z file with source path and password:

```rust
async_sevenz::compress_to_path_encrypted("examples/data/sample", "examples/data/sample.7z", "password".into()).expect("compress ok");
```

### Advanced Usage

#### Solid compression

Solid archives can in theory provide better compression rates, but decompressing a file needs all previous data to also
be decompressed.

```rust
use async_sevenz::*;

let mut writer = ArchiveWriter::create("dest.7z").expect("create writer ok");
writer.push_source_path("path/to/compress", | _ | true).expect("pack ok");
writer.finish().expect("compress ok");
```

#### Configure the compression methods

With encryption and lzma2 options:

```rust
use async_sevenz::*;

let mut writer = ArchiveWriter::create("dest.7z").expect("create writer ok");
writer.set_content_methods(vec![
    encoder_options::AesEncoderOptions::new("sevenz-rust".into()).into(),
    encoder_options::Lzma2Options::from_level(9).into(),
]);
writer.push_source_path("path/to/compress", | _ | true).expect("pack ok");
writer.finish().expect("compress ok");
```

### WASM support

WASM is supported, but you can't use the default features. We provide a "default_wasm" feature that contains
all default features with the needed changes to support WASM:

```bash
RUSTFLAGS='--cfg getrandom_backend="wasm_js"' cargo build --target wasm32-unknown-unknown --no-default-features --features=default_wasm
```

## Licence

Licensed under the [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0).

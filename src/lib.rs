//! This project is an async 7z compressor/decompressor written in pure Rust.
//!
//! This is a fork of [sevenz-rust2](https://github.com/hasenbanck/sevenz-rust2), then translated the api to async with [async-compression](https://crates.io/crates/async-compression) by AI.
//!
//! ## Supported Codecs & filters
//!
//! | Codec          | Decompression | Compression |
//! |----------------|---------------|-------------|
//! | COPY           | ✓             | ✓           |
//! | LZMA           | ✓             | ✓           |
//! | LZMA2          | ✓             | ✓           |
//! | BROTLI (*)     | ✓             | ✓           |
//! | BZIP2          | ✓             | ✓           |
//! | DEFLATE (*)    | ✓             | ✓           |
//! | PPMD           | ✓             | ✓           |
//! | LZ4 (*)        | ✓             | ✓           |
//! | ZSTD (*)       | ✓             | ✓           |
//!
//! (*) Require optional cargo feature.
//!
//! | Filter        | Decompression | Compression |
//! |---------------|---------------|-------------|
//! | BCJ X86       | ✓             | ✓           |
//! | BCJ ARM       | ✓             | ✓           |
//! | BCJ ARM64     | ✓             | ✓           |
//! | BCJ ARM_THUMB | ✓             | ✓           |
//! | BCJ RISC_V    | ✓             | ✓           |
//! | BCJ PPC       | ✓             | ✓           |
//! | BCJ SPARC     | ✓             | ✓           |
//! | BCJ IA64      | ✓             | ✓           |
//! | BCJ2          | ✓             |             |
//! | DELTA         | ✓             | ✓           |
//!
//! # Usage
//!
//! ```rust
//! use std::path::PathBuf;
//!
//! use tempfile::tempdir;
//!
//! use async_sevenz::decompress_file;
//!
//! let mut src = PathBuf::new();
//! src.push("examples/data/sample.7z");
//! let dest = tempdir().unwrap();
//! tokio::runtime::Runtime::new().unwrap().block_on(decompress_file(src, dest.path())).expect("complete");
//! ```
//!
//! ## Decompress an encrypted 7z file
//!
//! ```rust
//! # #[cfg(feature = "aes256")]
//! # {
//! use std::path::PathBuf;
//!
//! use tempfile::tempdir;
//!
//! use async_sevenz::decompress_file_with_password;
//!
//! let mut src = PathBuf::new();
//! src.push("tests/resources/encrypted.7z");
//! let dest = tempdir().unwrap();
//! tokio::runtime::Runtime::new().unwrap().block_on(decompress_file_with_password(src, dest.path(), "sevenz-rust".into()))
//!     .expect("complete");
//! # }
//! ```
//!
//! # Compression
//!
//! ```rust
//! # #[cfg(feature = "compress")]
//! # {
//! use std::path::PathBuf;
//!
//! use tempfile::tempdir;
//!
//! use async_sevenz::compress_to_path;
//!
//! let src = PathBuf::from("examples/data/sample");
//! let dest_dir = tempdir().unwrap();
//! let dest = dest_dir.path().join("sample.7z");
//! tokio::runtime::Runtime::new().unwrap().block_on(compress_to_path(src, &dest)).expect("compress ok");
//! # }
//! ```
//!
//! ## Compress with AES encryption
//!
//! ```rust
//! # #[cfg(all(feature = "compress", feature = "aes256"))]
//! # {
//! use std::path::PathBuf;
//!
//! use tempfile::tempdir;
//!
//! use async_sevenz::compress_to_path_encrypted;
//!
//! let src = PathBuf::from("examples/data/sample");
//! let dest_dir = tempdir().unwrap();
//! let dest = dest_dir.path().join("sample_encrypted.7z");
//! tokio::runtime::Runtime::new().unwrap().block_on(compress_to_path_encrypted(src, &dest, "sevenz-rust".into()))
//!     .expect("compress ok");
//! # }
//! ```
//!
//! ## Solid compression
//!
//! ```rust
//! # #[cfg(feature = "compress")]
//! # {
//! use async_sevenz::ArchiveWriter;
//!
//! tokio::runtime::Runtime::new().unwrap().block_on(async {
//!     let mut writer = ArchiveWriter::create_in_memory()
//!         .await
//!         .expect("create writer ok");
//!     writer
//!         .push_source_path("examples/data/sample", |_| async { true })
//!         .await
//!         .expect("pack ok");
//!     writer.finish().await.expect("compress ok");
//! });
//! # }
//! ```
//!
//! ## Configure the compression methods
//!
//! ```rust
//! # #[cfg(feature = "compress")]
//! # {
//! use async_sevenz::{ArchiveWriter, encoder_options};
//!
//! tokio::runtime::Runtime::new().unwrap().block_on(async {
//!     let mut writer = ArchiveWriter::create_in_memory()
//!         .await
//!         .expect("create writer ok");
//!     writer.set_content_methods(vec![
//!         encoder_options::AesEncoderOptions::new("sevenz-rust".into()).into(),
//!         encoder_options::Lzma2Options::from_level(9).into(),
//!     ]);
//!     writer
//!         .push_source_path("examples/data/sample", |_| async { true })
//!         .await
//!         .expect("pack ok");
//!     writer.finish().await.expect("compress ok");
//! });
//! # }
//! ```
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

#[cfg(target_arch = "wasm32")]
extern crate wasm_bindgen;

#[cfg(feature = "compress")]
mod encoder;
/// Encoding options when compressing.
#[cfg(feature = "compress")]
pub mod encoder_options;
mod encryption;
mod error;
mod reader;

#[cfg(feature = "compress")]
mod writer;

pub(crate) mod archive;
pub(crate) mod bitset;
pub(crate) mod block;
mod codec;
pub(crate) mod decoder;

mod time;
mod util;

use std::ops::{Deref, DerefMut};

pub use archive::*;
pub use block::*;
pub use encryption::Password;
pub use error::Error;
pub use reader::{ArchiveReader, BlockDecoder};
pub use time::NtTime;
#[cfg(all(feature = "compress", not(target_arch = "wasm32")))]
pub use util::compress::*;
#[cfg(not(target_arch = "wasm32"))]
pub use util::decompress::*;
#[cfg(target_arch = "wasm32")]
pub use util::wasm::*;
#[cfg(feature = "compress")]
pub use writer::*;

/// A trait for writers that finishes the stream on drop.
pub trait AutoFinish {
    /// Finish writing the stream without error handling.
    fn finish_ignore_error(self);
}

/// A wrapper around a writer that finishes the stream on drop.
pub struct AutoFinisher<T: AutoFinish>(Option<T>);

impl<T: AutoFinish> Drop for AutoFinisher<T> {
    fn drop(&mut self) {
        if let Some(writer) = self.0.take() {
            writer.finish_ignore_error();
        }
    }
}

impl<T: AutoFinish> Deref for AutoFinisher<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl<T: AutoFinish> DerefMut for AutoFinisher<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}

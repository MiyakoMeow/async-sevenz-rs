//! This project is a 7z compressor/decompressor written in pure Rust.
//!
//! This is a fork of the original, unmaintained sevenz-rust crate to continue the development
//! and maintenance.
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
#[cfg(feature = "util")]
mod util;

use std::ops::{Deref, DerefMut};

pub use archive::*;
pub use block::*;
pub use encryption::Password;
pub use error::Error;
pub use reader::{ArchiveReader, BlockDecoder};
pub use time::NtTime;
#[cfg(all(feature = "compress", feature = "util", not(target_arch = "wasm32")))]
pub use util::compress::*;
#[cfg(all(feature = "util", not(target_arch = "wasm32")))]
pub use util::decompress::*;
#[cfg(all(feature = "util", target_arch = "wasm32"))]
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

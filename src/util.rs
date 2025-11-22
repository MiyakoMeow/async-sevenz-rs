#[cfg(all(feature = "compress", not(target_arch = "wasm32")))]
pub(crate) mod compress;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod decompress;

#[cfg(target_arch = "wasm32")]
pub(crate) mod wasm;

use std::sync::Arc;

use super::*;
use crate::EncoderConfiguration;
#[derive(Debug, Clone, Default)]
pub(crate) struct UnpackInfo {
    pub(crate) blocks: Vec<BlockInfo>,
}

impl UnpackInfo {
    pub(crate) fn add(
        &mut self,
        methods: Arc<Vec<EncoderConfiguration>>,
        sizes: Vec<u64>,
        crc: u32,
    ) {
        self.blocks.push(BlockInfo {
            methods,
            sizes,
            crc,
            num_sub_unpack_streams: 1,
            ..Default::default()
        })
    }

    pub(crate) fn add_multiple(
        &mut self,
        methods: Arc<Vec<EncoderConfiguration>>,
        sizes: Vec<u64>,
        crc: u32,
        num_sub_unpack_streams: u64,
        sub_stream_sizes: Vec<u64>,
        sub_stream_crcs: Vec<u32>,
    ) {
        self.blocks.push(BlockInfo {
            methods,
            sizes,
            crc,
            num_sub_unpack_streams,
            sub_stream_crcs,
            sub_stream_sizes,
        })
    }

    pub(crate) async fn write_to<W: AsyncWrite + Unpin>(
        &mut self,
        header: &mut W,
    ) -> std::io::Result<()> {
        AsyncWriteExt::write_all(header, &[K_UNPACK_INFO]).await?;
        AsyncWriteExt::write_all(header, &[K_FOLDER]).await?;
        write_encoded_u64(header, self.blocks.len() as u64).await?;
        AsyncWriteExt::write_all(header, &[0]).await?;
        let mut cache = Vec::with_capacity(32);
        for block in self.blocks.iter() {
            block.write_to(header, &mut cache).await?;
        }
        AsyncWriteExt::write_all(header, &[K_CODERS_UNPACK_SIZE]).await?;
        for block in self.blocks.iter() {
            for size in block.sizes.iter().copied() {
                write_encoded_u64(header, size).await?;
            }
        }
        // 7zip doesn't write CRC values in the folder section of the unpack info. Instead,
        // it writes it only in the substreams info (even for non-solid archives).
        AsyncWriteExt::write_all(header, &[K_END]).await?;
        Ok(())
    }

    pub(crate) async fn write_substreams<W: AsyncWrite + Unpin>(
        &self,
        header: &mut W,
    ) -> std::io::Result<()> {
        AsyncWriteExt::write_all(header, &[K_SUB_STREAMS_INFO]).await?;

        // Only write K_NUM_UNPACK_STREAM if any folder has != 1 substream.
        let needs_num_unpack_stream = self.blocks.iter().any(|f| f.num_sub_unpack_streams != 1);

        if needs_num_unpack_stream {
            AsyncWriteExt::write_all(header, &[K_NUM_UNPACK_STREAM]).await?;
            for f in &self.blocks {
                write_encoded_u64(header, f.num_sub_unpack_streams).await?;
            }
        }

        // Only write K_SIZE if there are folders with > 1 substream.
        let needs_sizes = self.blocks.iter().any(|f| f.sub_stream_sizes.len() > 1);

        if needs_sizes {
            AsyncWriteExt::write_all(header, &[K_SIZE]).await?;
            for f in &self.blocks {
                if f.sub_stream_sizes.len() > 1 {
                    debug_assert_eq!(f.sub_stream_sizes.len(), f.num_sub_unpack_streams as usize);

                    // Write N-1 sizes (last size is calculated).
                    for i in 0..f.sub_stream_sizes.len() - 1 {
                        let size = f.sub_stream_sizes[i];
                        write_encoded_u64(header, size).await?;
                    }
                }
            }
        }

        // We always write the CRC values in the substreams info.
        let mut crcs_to_write = Vec::new();
        for f in &self.blocks {
            if f.num_sub_unpack_streams > 1 {
                // Multiple substreams - write all CRCs.
                for &crc in &f.sub_stream_crcs {
                    crcs_to_write.push(crc);
                }
            } else if f.num_sub_unpack_streams == 1 {
                // Single substream - write CRC here and not in the folder section.
                match f.sub_stream_crcs.first() {
                    None => {
                        crcs_to_write.push(f.crc);
                    }
                    Some(crc) => {
                        crcs_to_write.push(*crc);
                    }
                };
            }
        }

        if !crcs_to_write.is_empty() {
            AsyncWriteExt::write_all(header, &[K_CRC]).await?;
            AsyncWriteExt::write_all(header, &[1]).await?; // all CRCs defined.
            for crc in crcs_to_write {
                AsyncWriteExt::write_all(header, &crc.to_le_bytes()).await?;
            }
        }

        AsyncWriteExt::write_all(header, &[K_END]).await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct BlockInfo {
    pub(crate) methods: Arc<Vec<EncoderConfiguration>>,
    pub(crate) sizes: Vec<u64>,
    pub(crate) crc: u32,
    pub(crate) num_sub_unpack_streams: u64,
    pub(crate) sub_stream_sizes: Vec<u64>,
    pub(crate) sub_stream_crcs: Vec<u32>,
}

impl BlockInfo {
    pub(crate) async fn write_to<W: AsyncWrite + Unpin>(
        &self,
        header: &mut W,
        cache: &mut Vec<u8>,
    ) -> std::io::Result<()> {
        cache.clear();
        let mut num_coders = 0;
        for mc in self.methods.iter() {
            num_coders += 1;
            self.write_single_codec(mc, cache).await?;
        }
        write_encoded_u64(header, num_coders as u64).await?;
        AsyncWriteExt::write_all(header, cache).await?;
        for i in 0..num_coders - 1 {
            write_encoded_u64(header, i as u64 + 1).await?;
            write_encoded_u64(header, i as u64).await?;
        }
        Ok(())
    }

    async fn write_single_codec<W: AsyncWrite + Unpin>(
        &self,
        mc: &EncoderConfiguration,
        out: &mut W,
    ) -> std::io::Result<()> {
        let id = mc.method.id();
        let mut temp = [0u8; 256];
        let props = encoder::get_options_as_properties(mc.method, mc.options.as_ref(), &mut temp);
        let mut codec_flags = id.len() as u8;
        if !props.is_empty() {
            codec_flags |= 0x20;
        }
        AsyncWriteExt::write_all(out, &[codec_flags]).await?;
        AsyncWriteExt::write_all(out, id).await?;
        if !props.is_empty() {
            AsyncWriteExt::write_all(out, &[props.len() as u8]).await?;
            AsyncWriteExt::write_all(out, props).await?;
        }
        Ok(())
    }
}

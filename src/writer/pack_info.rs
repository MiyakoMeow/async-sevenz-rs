use super::*;

#[derive(Debug, Default, Clone)]
pub(crate) struct PackInfo {
    pub(crate) crcs: Vec<u32>,
    pub(crate) sizes: Vec<u64>,
    pub(crate) pos: u64,
}

impl PackInfo {
    pub(crate) async fn write_to<W: AsyncWrite + Unpin>(
        &mut self,
        header: &mut W,
    ) -> std::io::Result<()> {
        AsyncWriteExt::write_all(header, &[K_PACK_INFO]).await?;
        write_encoded_u64(header, self.pos).await?;
        write_encoded_u64(header, self.len() as u64).await?;
        AsyncWriteExt::write_all(header, &[K_SIZE]).await?;
        for size in &self.sizes {
            write_encoded_u64(header, *size).await?;
        }
        AsyncWriteExt::write_all(header, &[K_CRC]).await?;
        let all_crc_defined = self.crcs.iter().all(|f| *f != 0);
        if all_crc_defined {
            AsyncWriteExt::write_all(header, &[1]).await?; // all defined
            for crc in self.crcs.iter() {
                AsyncWriteExt::write_all(header, &crc.to_le_bytes()).await?;
            }
        } else {
            AsyncWriteExt::write_all(header, &[0]).await?; // not all defined
            let mut crc_define_bits = BitSet::with_capacity(self.crcs.len());

            for (i, crc) in self.crcs.iter().cloned().enumerate() {
                if crc != 0 {
                    crc_define_bits.insert(i);
                }
            }
            let temp = bitset_to_bytes(&crc_define_bits, self.crcs.len());
            AsyncWriteExt::write_all(header, &temp).await?;
        }

        AsyncWriteExt::write_all(header, &[K_END]).await?;
        Ok(())
    }
}

impl PackInfo {
    #[inline]
    pub(crate) fn add_stream(&mut self, size: u64, crc: u32) {
        self.sizes.push(size);
        self.crcs.push(crc);
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.sizes.len()
    }
}

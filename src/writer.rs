mod counting_writer;
#[cfg(not(target_arch = "wasm32"))]
mod lazy_file_reader;
mod pack_info;
mod seq_reader;
mod source_reader;
mod unpack_info;

use futures::io::{
    AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt, AsyncWrite, AsyncWriteExt, SeekFrom,
};
use std::{cell::Cell, rc::Rc, sync::Arc};

pub(crate) use counting_writer::CountingWriter;
use crc32fast::Hasher;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) use self::lazy_file_reader::LazyFileReader;
pub(crate) use self::seq_reader::SeqReader;
pub use self::source_reader::SourceReader;
use self::{pack_info::PackInfo, unpack_info::UnpackInfo};
use crate::{ArchiveEntry, AutoFinish, AutoFinisher, Error, archive::*, bitset::BitSet, encoder};

macro_rules! write_times {
    ($fn_name:tt, $nid:expr, $has_time:tt, $time:tt) => {
        async fn $fn_name<H: AsyncWrite + Unpin>(&self, header: &mut H) -> std::io::Result<()> {
            let mut num = 0;
            for entry in self.files.iter() {
                if entry.$has_time {
                    num += 1;
                }
            }
            if num > 0 {
                AsyncWriteExt::write_all(header, &[$nid]).await?;
                let mut temp: Vec<u8> = Vec::with_capacity(128);
                if num != self.files.len() {
                    temp.push(0);
                    let mut times = BitSet::with_capacity(self.files.len());
                    for i in 0..self.files.len() {
                        if self.files[i].$has_time {
                            times.insert(i);
                        }
                    }
                    let bits = bitset_to_bytes(&times, self.files.len());
                    temp.extend_from_slice(&bits);
                } else {
                    temp.push(1);
                }
                temp.push(0);
                for file in self.files.iter() {
                    if file.$has_time {
                        vec_push_le_u64(&mut temp, (file.$time).into());
                    }
                }
                write_encoded_u64(header, temp.len() as u64).await?;
                AsyncWriteExt::write_all(header, &temp).await?;
            }
            Ok(())
        }
    };
    ($fn_name:tt, $nid:expr, $has_time:tt, $time:tt, write_u32) => {
        async fn $fn_name<H: AsyncWrite + Unpin>(&self, header: &mut H) -> std::io::Result<()> {
            let mut num = 0;
            for entry in self.files.iter() {
                if entry.$has_time {
                    num += 1;
                }
            }
            if num > 0 {
                AsyncWriteExt::write_all(header, &[$nid]).await?;
                let mut temp: Vec<u8> = Vec::with_capacity(128);
                if num != self.files.len() {
                    temp.push(0);
                    let mut times = BitSet::with_capacity(self.files.len());
                    for i in 0..self.files.len() {
                        if self.files[i].$has_time {
                            times.insert(i);
                        }
                    }
                    let bits = bitset_to_bytes(&times, self.files.len());
                    temp.extend_from_slice(&bits);
                } else {
                    temp.push(1);
                }
                temp.push(0);
                for file in self.files.iter() {
                    if file.$has_time {
                        vec_push_le_u32(&mut temp, file.$time);
                    }
                }
                write_encoded_u64(header, temp.len() as u64).await?;
                AsyncWriteExt::write_all(header, &temp).await?;
            }
            Ok(())
        }
    };
}

type Result<T> = std::result::Result<T, Error>;

/// Writes a 7z archive file.
pub struct ArchiveWriter<W: AsyncWrite + AsyncSeek + Unpin> {
    output: W,
    files: Vec<ArchiveEntry>,
    content_methods: Arc<Vec<EncoderConfiguration>>,
    pack_info: PackInfo,
    unpack_info: UnpackInfo,
    encrypt_header: bool,
}

#[cfg(not(target_arch = "wasm32"))]
impl ArchiveWriter<futures::io::Cursor<Vec<u8>>> {
    /// 创建一个基于内存缓冲的 7z 写入器。
    ///
    /// 返回使用 `Vec<u8>` 作为底层存储的 `ArchiveWriter`，适合测试或无需落盘的场景。
    pub async fn create_in_memory() -> Result<Self> {
        let cursor = futures::io::Cursor::new(Vec::<u8>::new());
        Self::new(cursor).await
    }
}

impl<W: AsyncWrite + AsyncSeek + Unpin> ArchiveWriter<W> {
    /// Prepares writer to write a 7z archive to.
    pub async fn new(mut writer: W) -> Result<Self> {
        AsyncSeekExt::seek(&mut writer, SeekFrom::Start(SIGNATURE_HEADER_SIZE)).await?;

        Ok(Self {
            output: writer,
            files: Default::default(),
            content_methods: Arc::new(vec![EncoderConfiguration::new(EncoderMethod::LZMA2)]),
            pack_info: Default::default(),
            unpack_info: Default::default(),
            encrypt_header: true,
        })
    }

    /// Returns a wrapper around `self` that will finish the stream on drop.
    pub fn auto_finish(self) -> AutoFinisher<Self> {
        AutoFinisher(Some(self))
    }

    /// Sets the default compression methods to use for entry data. Default is LZMA2.
    pub fn set_content_methods(&mut self, content_methods: Vec<EncoderConfiguration>) -> &mut Self {
        if content_methods.is_empty() {
            return self;
        }
        self.content_methods = Arc::new(content_methods);
        self
    }

    /// Whether to enable the encryption of the -header. Default is `true`.
    pub fn set_encrypt_header(&mut self, enabled: bool) {
        self.encrypt_header = enabled;
    }

    /// Non-solid compression - Adds an archive `entry` with data from `reader`.
    ///
    /// # Example
    /// ```no_run
    /// use std::io::Cursor;
    /// use std::path::Path;
    /// use async_sevenz::*;
    /// let mut sz = tokio::runtime::Runtime::new().unwrap().block_on(ArchiveWriter::create_in_memory()).expect("create writer ok");
    /// let src = Path::new("path/to/source.txt");
    /// let name = "source.txt".to_string();
    /// let entry = tokio::runtime::Runtime::new().unwrap().block_on(async {
    ///     sz
    ///         .push_archive_entry(
    ///             ArchiveEntry::from_path(&src, name).await,
    ///             Some(futures::io::Cursor::new(&b"example"[..])),
    ///         )
    ///         .await
    ///         .expect("ok")
    /// });
    /// let compressed_size = entry.compressed_size;
    /// let _cursor = tokio::runtime::Runtime::new().unwrap().block_on(sz.finish()).expect("done");
    /// ```
    pub async fn push_archive_entry<R: AsyncRead + Unpin>(
        &mut self,
        mut entry: ArchiveEntry,
        reader: Option<R>,
    ) -> Result<&ArchiveEntry> {
        if !entry.is_directory {
            if let Some(mut r) = reader {
                let mut compressed_len = 0;
                let mut compressed = CompressWrapWriter::new(&mut self.output, &mut compressed_len);

                let mut more_sizes: Vec<Rc<Cell<usize>>> =
                    Vec::with_capacity(self.content_methods.len() - 1);

                let (crc, size) = {
                    let mut w = Self::create_writer(
                        &self.content_methods,
                        &mut compressed,
                        &mut more_sizes,
                    )?;
                    let mut write_len = 0;
                    let mut w = CompressWrapWriter::new(&mut w, &mut write_len);
                    let mut buf = [0u8; 4096];
                    loop {
                        let n = AsyncReadExt::read(&mut r, &mut buf).await.map_err(|e| {
                            Error::io_msg(e, format!("Encode entry:{}", entry.name()))
                        })?;
                        if n == 0 {
                            break;
                        }
                        AsyncWriteExt::write_all(&mut w, &buf[..n])
                            .await
                            .map_err(|e| {
                                Error::io_msg(e, format!("Encode entry:{}", entry.name()))
                            })?;
                    }
                    AsyncWriteExt::flush(&mut w)
                        .await
                        .map_err(|e| Error::io_msg(e, format!("Encode entry:{}", entry.name())))?;
                    AsyncWriteExt::write(&mut w, &[])
                        .await
                        .map_err(|e| Error::io_msg(e, format!("Encode entry:{}", entry.name())))?;

                    (w.crc_value(), write_len)
                };
                let compressed_crc = compressed.crc_value();
                entry.has_stream = true;
                entry.size = size as u64;
                entry.crc = crc as u64;
                entry.has_crc = true;
                entry.compressed_crc = compressed_crc as u64;
                entry.compressed_size = compressed_len as u64;
                self.pack_info
                    .add_stream(compressed_len as u64, compressed_crc);

                let mut sizes = Vec::with_capacity(more_sizes.len() + 1);
                sizes.extend(more_sizes.iter().map(|s| s.get() as u64));
                sizes.push(size as u64);

                self.unpack_info
                    .add(self.content_methods.clone(), sizes, crc);

                self.files.push(entry);
                return Ok(self.files.last().unwrap());
            }
        }
        entry.has_stream = false;
        entry.size = 0;
        entry.compressed_size = 0;
        entry.has_crc = false;
        self.files.push(entry);
        Ok(self.files.last().unwrap())
    }

    /// Solid compression - packs `entries` into one pack.
    ///
    /// # Panics
    /// * If `entries`'s length not equals to `reader.reader_len()`
    pub async fn push_archive_entries<R: AsyncRead + Unpin>(
        &mut self,
        entries: Vec<ArchiveEntry>,
        reader: Vec<SourceReader<R>>,
    ) -> Result<&mut Self> {
        let mut entries = entries;
        let mut r = SeqReader::new(reader);
        assert_eq!(r.reader_len(), entries.len());
        let mut compressed_len = 0;
        let mut compressed = CompressWrapWriter::new(&mut self.output, &mut compressed_len);
        let content_methods = &self.content_methods;
        let mut more_sizes: Vec<Rc<Cell<usize>>> = Vec::with_capacity(content_methods.len() - 1);

        let (crc, size) = {
            let mut w = Self::create_writer(content_methods, &mut compressed, &mut more_sizes)?;
            let mut write_len = 0;
            let mut w = CompressWrapWriter::new(&mut w, &mut write_len);
            let mut buf = [0u8; 4096];

            fn entries_names(entries: &[ArchiveEntry]) -> String {
                let mut names = String::with_capacity(512);
                for ele in entries.iter() {
                    names.push_str(&ele.name);
                    names.push(';');
                    if names.len() > 512 {
                        break;
                    }
                }
                names
            }

            loop {
                let n = AsyncReadExt::read(&mut r, &mut buf).await.map_err(|e| {
                    Error::io_msg(e, format!("Encode entries:{}", entries_names(&entries)))
                })?;
                if n == 0 {
                    break;
                }
                AsyncWriteExt::write_all(&mut w, &buf[..n])
                    .await
                    .map_err(|e| {
                        Error::io_msg(e, format!("Encode entries:{}", entries_names(&entries)))
                    })?;
            }
            AsyncWriteExt::flush(&mut w).await.map_err(|e| {
                let mut names = String::with_capacity(512);
                for ele in entries.iter() {
                    names.push_str(&ele.name);
                    names.push(';');
                    if names.len() > 512 {
                        break;
                    }
                }
                Error::io_msg(e, format!("Encode entry:{names}"))
            })?;
            AsyncWriteExt::write(&mut w, &[]).await.map_err(|e| {
                Error::io_msg(e, format!("Encode entry:{}", entries_names(&entries)))
            })?;

            (w.crc_value(), write_len)
        };
        let compressed_crc = compressed.crc_value();
        let mut sub_stream_crcs = Vec::with_capacity(entries.len());
        let mut sub_stream_sizes = Vec::with_capacity(entries.len());
        for i in 0..entries.len() {
            let entry = &mut entries[i];
            let ri = &r[i];
            entry.crc = ri.crc_value() as u64;
            entry.size = ri.read_count() as u64;
            sub_stream_crcs.push(entry.crc as u32);
            sub_stream_sizes.push(entry.size);
            entry.has_crc = true;
        }

        self.pack_info
            .add_stream(compressed_len as u64, compressed_crc);

        let mut sizes = Vec::with_capacity(more_sizes.len() + 1);
        sizes.extend(more_sizes.iter().map(|s| s.get() as u64));
        sizes.push(size as u64);

        self.unpack_info.add_multiple(
            content_methods.clone(),
            sizes,
            crc,
            entries.len() as u64,
            sub_stream_sizes,
            sub_stream_crcs,
        );

        self.files.extend(entries);
        Ok(self)
    }

    fn create_writer<'a, O: AsyncWrite + Unpin + 'a>(
        methods: &[EncoderConfiguration],
        out: O,
        more_sized: &mut Vec<Rc<Cell<usize>>>,
    ) -> Result<Box<dyn AsyncWrite + Unpin + 'a>> {
        let mut encoder: Box<dyn AsyncWrite + Unpin> = Box::new(out);
        let mut first = true;
        for mc in methods.iter() {
            if !first {
                let counting = CountingWriter::new(encoder);
                more_sized.push(counting.counting());
                encoder = Box::new(encoder::add_encoder(counting, mc)?);
            } else {
                let counting = CountingWriter::new(encoder);
                encoder = Box::new(encoder::add_encoder(counting, mc)?);
            }
            first = false;
        }
        Ok(encoder)
    }

    /// Finishes the compression.
    pub async fn finish(mut self) -> std::io::Result<W> {
        let mut cursor = futures::io::Cursor::new(Vec::with_capacity(64 * 1024));
        self.write_encoded_header(&mut cursor).await?;
        let header = cursor.into_inner();
        let header_pos = AsyncSeekExt::stream_position(&mut self.output).await?;
        AsyncWriteExt::write_all(&mut self.output, &header).await?;
        let crc32 = crc32fast::hash(&header);
        let mut hh = [0u8; SIGNATURE_HEADER_SIZE as usize];
        hh[0..SEVEN_Z_SIGNATURE.len()].copy_from_slice(SEVEN_Z_SIGNATURE);
        hh[6] = 0;
        hh[7] = 4;
        hh[8..12].copy_from_slice(&0u32.to_le_bytes());
        let start_header_offset_le = (header_pos - SIGNATURE_HEADER_SIZE).to_le_bytes();
        hh[12..20].copy_from_slice(&start_header_offset_le);
        let start_header_len_le = ((header.len() as u64) & 0xFFFF_FFFF).to_le_bytes();
        hh[20..28].copy_from_slice(&start_header_len_le);
        hh[28..32].copy_from_slice(&crc32.to_le_bytes());
        let crc32 = crc32fast::hash(&hh[12..]);
        hh[8..12].copy_from_slice(&crc32.to_le_bytes());

        AsyncSeekExt::seek(&mut self.output, SeekFrom::Start(0)).await?;
        AsyncWriteExt::write_all(&mut self.output, &hh).await?;
        AsyncWriteExt::flush(&mut self.output).await?;
        Ok(self.output)
    }

    async fn write_header<H: AsyncWrite + Unpin>(&mut self, header: &mut H) -> std::io::Result<()> {
        AsyncWriteExt::write_all(header, &[K_HEADER]).await?;
        AsyncWriteExt::write_all(header, &[K_MAIN_STREAMS_INFO]).await?;
        self.write_streams_info(header).await?;
        self.write_files_info(header).await?;
        AsyncWriteExt::write_all(header, &[K_END]).await?;
        Ok(())
    }

    async fn write_encoded_header<H: AsyncWrite + Unpin>(
        &mut self,
        header: &mut H,
    ) -> std::io::Result<()> {
        let mut raw_header_cursor = futures::io::Cursor::new(Vec::with_capacity(64 * 1024));
        self.write_header(&mut raw_header_cursor).await?;
        let raw_header = raw_header_cursor.into_inner();
        let mut pack_info = PackInfo::default();

        let position = AsyncSeekExt::stream_position(&mut self.output).await?;
        let pos = position - SIGNATURE_HEADER_SIZE;
        pack_info.pos = pos;

        let mut more_sizes = vec![];
        let size = raw_header.len() as u64;
        let crc32 = crc32fast::hash(&raw_header);
        let mut methods = vec![];

        if self.encrypt_header {
            for conf in self.content_methods.iter() {
                if conf.method.id() == EncoderMethod::AES256_SHA256.id() {
                    methods.push(conf.clone());
                    break;
                }
            }
        }

        methods.push(EncoderConfiguration::new(EncoderMethod::LZMA2));

        let methods = Arc::new(methods);

        let mut encoded_cursor = futures::io::Cursor::new(Vec::with_capacity(size as usize / 2));

        let mut compress_size = 0;
        let mut compressed = CompressWrapWriter::new(&mut encoded_cursor, &mut compress_size);
        {
            let mut encoder = Self::create_writer(&methods, &mut compressed, &mut more_sizes)
                .map_err(std::io::Error::other)?;
            AsyncWriteExt::write_all(&mut encoder, &raw_header).await?;
            AsyncWriteExt::flush(&mut encoder).await?;
            let _ = AsyncWriteExt::write(&mut encoder, &[]).await?;
        }

        let compress_crc = compressed.crc_value();
        let compress_size = *compressed.bytes_written;
        if compress_size as u64 + 20 >= size {
            AsyncWriteExt::write_all(header, &raw_header).await?;
            return Ok(());
        }
        let encoded_data = encoded_cursor.into_inner();
        AsyncWriteExt::write_all(&mut self.output, &encoded_data[..compress_size]).await?;

        pack_info.add_stream(compress_size as u64, compress_crc);

        let mut unpack_info = UnpackInfo::default();
        let mut sizes = Vec::with_capacity(1 + more_sizes.len());
        sizes.extend(more_sizes.iter().map(|s| s.get() as u64));
        sizes.push(size);
        unpack_info.add(methods, sizes, crc32);

        AsyncWriteExt::write_all(header, &[K_ENCODED_HEADER]).await?;

        pack_info.write_to(header).await?;
        unpack_info.write_to(header).await?;
        unpack_info.write_substreams(header).await?;

        AsyncWriteExt::write_all(header, &[K_END]).await?;

        Ok(())
    }

    async fn write_streams_info<H: AsyncWrite + Unpin>(
        &mut self,
        header: &mut H,
    ) -> std::io::Result<()> {
        if self.pack_info.len() > 0 {
            self.pack_info.write_to(header).await?;
            self.unpack_info.write_to(header).await?;
        }
        self.unpack_info.write_substreams(header).await?;

        AsyncWriteExt::write_all(header, &[K_END]).await?;
        Ok(())
    }

    async fn write_files_info<H: AsyncWrite + Unpin>(&self, header: &mut H) -> std::io::Result<()> {
        AsyncWriteExt::write_all(header, &[K_FILES_INFO]).await?;
        write_encoded_u64(header, self.files.len() as u64).await?;
        self.write_file_empty_streams(header).await?;
        self.write_file_empty_files(header).await?;
        self.write_file_anti_items(header).await?;
        self.write_file_names(header).await?;
        self.write_file_ctimes(header).await?;
        self.write_file_atimes(header).await?;
        self.write_file_mtimes(header).await?;
        self.write_file_windows_attrs(header).await?;
        AsyncWriteExt::write_all(header, &[K_END]).await?;
        Ok(())
    }

    async fn write_file_empty_streams<H: AsyncWrite + Unpin>(
        &self,
        header: &mut H,
    ) -> std::io::Result<()> {
        let mut has_empty = false;
        for entry in self.files.iter() {
            if !entry.has_stream {
                has_empty = true;
                break;
            }
        }
        if has_empty {
            AsyncWriteExt::write_all(header, &[K_EMPTY_STREAM]).await?;
            let mut bitset = BitSet::with_capacity(self.files.len());
            for (i, entry) in self.files.iter().enumerate() {
                if !entry.has_stream {
                    bitset.insert(i);
                }
            }
            let temp = bitset_to_bytes(&bitset, self.files.len());
            write_encoded_u64(header, temp.len() as u64).await?;
            AsyncWriteExt::write_all(header, &temp).await?;
        }
        Ok(())
    }

    async fn write_file_empty_files<H: AsyncWrite + Unpin>(
        &self,
        header: &mut H,
    ) -> std::io::Result<()> {
        let mut has_empty = false;
        let mut empty_stream_counter = 0;
        let mut bitset = BitSet::new();
        for entry in self.files.iter() {
            if !entry.has_stream {
                let is_dir = entry.is_directory();
                has_empty |= !is_dir;
                if !is_dir {
                    bitset.insert(empty_stream_counter);
                }
                empty_stream_counter += 1;
            }
        }
        if has_empty {
            AsyncWriteExt::write_all(header, &[K_EMPTY_FILE]).await?;

            let temp = bitset_to_bytes(&bitset, empty_stream_counter);
            write_encoded_u64(header, temp.len() as u64).await?;
            AsyncWriteExt::write_all(header, &temp).await?;
        }
        Ok(())
    }

    async fn write_file_anti_items<H: AsyncWrite + Unpin>(
        &self,
        header: &mut H,
    ) -> std::io::Result<()> {
        let mut has_anti = false;
        let mut counter = 0;
        let mut bitset = BitSet::new();
        for entry in self.files.iter() {
            if !entry.has_stream {
                let is_anti = entry.is_anti_item();
                has_anti |= !is_anti;
                if !is_anti {
                    bitset.insert(counter);
                }
                counter += 1;
            }
        }
        if has_anti {
            AsyncWriteExt::write_all(header, &[K_ANTI]).await?;

            let temp = bitset_to_bytes(&bitset, counter);
            write_encoded_u64(header, temp.len() as u64).await?;
            AsyncWriteExt::write_all(header, &temp).await?;
        }
        Ok(())
    }

    async fn write_file_names<H: AsyncWrite + Unpin>(&self, header: &mut H) -> std::io::Result<()> {
        AsyncWriteExt::write_all(header, &[K_NAME]).await?;
        let mut temp: Vec<u8> = Vec::with_capacity(128);
        temp.push(0);
        for file in self.files.iter() {
            for c in file.name().encode_utf16() {
                temp.extend_from_slice(&c.to_le_bytes());
            }
            temp.extend_from_slice(&[0u8; 2]);
        }
        write_encoded_u64(header, temp.len() as u64).await?;
        AsyncWriteExt::write_all(header, &temp).await?;
        Ok(())
    }

    write_times!(
        write_file_ctimes,
        K_C_TIME,
        has_creation_date,
        creation_date
    );
    write_times!(write_file_atimes, K_A_TIME, has_access_date, access_date);
    write_times!(
        write_file_mtimes,
        K_M_TIME,
        has_last_modified_date,
        last_modified_date
    );
    write_times!(
        write_file_windows_attrs,
        K_WIN_ATTRIBUTES,
        has_windows_attributes,
        windows_attributes,
        write_u32
    );
}

impl<W: AsyncWrite + AsyncSeek + Unpin> AutoFinish for ArchiveWriter<W> {
    fn finish_ignore_error(self) {
        let _ = async_io::block_on(self.finish());
    }
}

pub(crate) async fn write_encoded_u64<W: AsyncWrite + Unpin>(
    header: &mut W,
    mut value: u64,
) -> std::io::Result<()> {
    let mut first = 0u64;
    let mut mask = 0x80u64;
    let mut i = 0u8;
    while (i as usize) < 8 {
        if value < (1u64 << (7 * (i as usize + 1))) {
            first |= value >> (8 * i as usize);
            break;
        }
        first |= mask;
        mask >>= 1;
        i += 1;
    }
    AsyncWriteExt::write_all(header, &[(first & 0xFF) as u8]).await?;
    while i > 0 {
        AsyncWriteExt::write_all(header, &[(value & 0xFF) as u8]).await?;
        value >>= 8;
        i -= 1;
    }
    Ok(())
}

fn vec_push_le_u64(buf: &mut Vec<u8>, value: u64) {
    buf.extend_from_slice(&value.to_le_bytes());
}

fn vec_push_le_u32(buf: &mut Vec<u8>, value: u32) {
    buf.extend_from_slice(&value.to_le_bytes());
}

pub(crate) fn bitset_to_bytes(bs: &BitSet, capacity: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity((capacity / 8).saturating_add(1));
    let mut cache = 0u8;
    let mut shift: i32 = 7;
    for i in 0..capacity {
        let set = if bs.contains(i) { 1 } else { 0 };
        cache |= (set as u8) << shift;
        shift -= 1;
        if shift < 0 {
            out.push(cache);
            shift = 7;
            cache = 0;
        }
    }
    if shift != 7 {
        out.push(cache);
    }
    out
}

struct CompressWrapWriter<'a, W> {
    writer: W,
    crc: Hasher,
    cache: Vec<u8>,
    bytes_written: &'a mut usize,
}

impl<'a, W> CompressWrapWriter<'a, W> {
    pub fn new(writer: W, bytes_written: &'a mut usize) -> Self {
        Self {
            writer,
            crc: Hasher::new(),
            cache: Vec::with_capacity(8192),
            bytes_written,
        }
    }

    pub fn crc_value(&mut self) -> u32 {
        let crc = std::mem::replace(&mut self.crc, Hasher::new());
        crc.finalize()
    }
}

/* removed sync Write impl to eliminate synchronous bridging */

impl<W: AsyncWrite + Unpin> AsyncWrite for CompressWrapWriter<'_, W> {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        let this = &mut *self;
        this.cache.resize(buf.len(), Default::default());
        let poll = std::pin::Pin::new(&mut this.writer).poll_write(cx, buf);
        if let std::task::Poll::Ready(Ok(len)) = &poll {
            this.crc.update(&buf[..*len]);
            *this.bytes_written += *len;
        }
        poll
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.writer).poll_flush(cx)
    }

    fn poll_close(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.writer).poll_close(cx)
    }
}

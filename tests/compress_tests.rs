#[cfg(feature = "compress")]
use async_sevenz::encoder_options::*;
#[cfg(feature = "compress")]
use async_sevenz::*;
#[cfg(feature = "compress")]
use std::hash::{Hash, Hasher};
#[cfg(feature = "compress")]
use tempfile::*;

#[cfg(feature = "compress")]
#[tokio::test]
async fn compress_empty_file() {
    let temp_dir = tempdir().unwrap();
    let source = temp_dir.path().join("empty.txt");
    async_fs::write(&source, &[]).await.unwrap();
    let dest = temp_dir.path().join("empty.7z");
    compress_to_path(source, &dest).await.expect("compress ok");

    let decompress_dest = temp_dir.path().join("decompress");
    decompress_file(dest, &decompress_dest)
        .await
        .expect("decompress ok");
    assert!(decompress_dest.exists());
    let decompress_file = decompress_dest.join("empty.txt");
    assert!(decompress_file.exists());

    assert_eq!(
        async_fs::read_to_string(&decompress_file).await.unwrap(),
        ""
    );
}

#[cfg(feature = "compress")]
#[tokio::test]
async fn compress_one_file_with_content() {
    let temp_dir = tempdir().unwrap();
    let source = temp_dir.path().join("file1.txt");
    async_fs::write(&source, "file1 with content")
        .await
        .unwrap();
    let dest = temp_dir.path().join("file1.7z");
    compress_to_path(source, &dest).await.expect("compress ok");

    let decompress_dest = temp_dir.path().join("decompress");
    decompress_file(dest, &decompress_dest)
        .await
        .expect("decompress ok");
    assert!(decompress_dest.exists());
    let decompress_file = decompress_dest.join("file1.txt");
    assert!(decompress_file.exists());

    assert_eq!(
        async_fs::read_to_string(&decompress_file).await.unwrap(),
        "file1 with content"
    );
}

#[cfg(feature = "compress")]
#[tokio::test]
async fn compress_empty_folder() {
    let temp_dir = tempdir().unwrap();
    let folder = temp_dir.path().join("folder");
    async_fs::create_dir(&folder).await.unwrap();
    let dest = temp_dir.path().join("folder.7z");
    compress_to_path(&folder, &dest).await.expect("compress ok");

    let decompress_dest = temp_dir.path().join("decompress");
    decompress_file(dest, &decompress_dest)
        .await
        .expect("decompress ok");
    assert!(decompress_dest.exists());
    assert!(decompress_dest.read_dir().unwrap().next().is_none());
}

#[cfg(feature = "compress")]
#[tokio::test]
async fn compress_folder_with_one_file() {
    let temp_dir = tempdir().unwrap();
    let folder = temp_dir.path().join("folder");
    async_fs::create_dir(&folder).await.unwrap();
    async_fs::write(folder.join("file1.txt"), "file1 with content")
        .await
        .unwrap();
    let dest = temp_dir.path().join("folder.7z");
    compress_to_path(&folder, &dest).await.expect("compress ok");

    let decompress_dest = temp_dir.path().join("decompress");
    decompress_file(dest, &decompress_dest)
        .await
        .expect("decompress ok");
    assert!(decompress_dest.exists());
    let decompress_file = decompress_dest.join("file1.txt");
    assert!(decompress_file.exists());

    assert_eq!(
        async_fs::read_to_string(&decompress_file).await.unwrap(),
        "file1 with content"
    );
}

#[cfg(feature = "compress")]
#[tokio::test]
async fn compress_folder_with_multi_file() {
    let temp_dir = tempdir().unwrap();
    let folder = temp_dir.path().join("folder");
    async_fs::create_dir(&folder).await.unwrap();
    let mut files = Vec::with_capacity(100);
    let mut contents = Vec::with_capacity(100);
    for i in 1..=100 {
        let name = format!("file{i}.txt");
        let content = format!("file{i} with content");
        async_fs::write(folder.join(&name), &content).await.unwrap();
        files.push(name);
        contents.push(content);
    }
    let dest = temp_dir.path().join("folder.7z");
    compress_to_path(&folder, &dest).await.expect("compress ok");

    let decompress_dest = temp_dir.path().join("decompress");
    decompress_file(dest, &decompress_dest)
        .await
        .expect("decompress ok");
    assert!(decompress_dest.exists());
    for i in 0..files.len() {
        let name = &files[i];
        let content = &contents[i];
        let decompress_file = decompress_dest.join(name);
        assert!(decompress_file.exists());
        assert_eq!(
            &async_fs::read_to_string(&decompress_file).await.unwrap(),
            content
        );
    }
}

#[cfg(feature = "compress")]
#[tokio::test]
async fn compress_folder_with_nested_folder() {
    let temp_dir = tempdir().unwrap();
    let folder = temp_dir.path().join("folder");
    let inner = folder.join("a/b/c");
    async_fs::create_dir_all(&inner).await.unwrap();
    async_fs::write(inner.join("file1.txt"), "file1 with content")
        .await
        .unwrap();
    let dest = temp_dir.path().join("folder.7z");
    compress_to_path(&folder, &dest).await.expect("compress ok");

    let decompress_dest = temp_dir.path().join("decompress");
    decompress_file(dest, &decompress_dest)
        .await
        .expect("decompress ok");
    assert!(decompress_dest.exists());
    let decompress_file = decompress_dest.join("a/b/c/file1.txt");
    assert!(decompress_file.exists());

    assert_eq!(
        async_fs::read_to_string(&decompress_file).await.unwrap(),
        "file1 with content"
    );
}

#[cfg(all(feature = "compress", feature = "aes256"))]
#[tokio::test]
async fn compress_one_file_with_random_content_encrypted() {
    use rand::Rng;
    for _ in 0..10 {
        let temp_dir = tempdir().unwrap();
        let source = temp_dir.path().join("file1.txt");
        let mut rng = rand::rng();
        let mut content = String::with_capacity(rng.random_range(1..10240));

        for _ in 0..content.capacity() {
            let c = rng.random_range(' '..'~');
            content.push(c);
        }
        async_fs::write(&source, &content).await.unwrap();
        let dest = temp_dir.path().join("file1.7z");

        compress_to_path_encrypted(source, &dest, "rust".into())
            .await
            .expect("compress ok");

        let decompress_dest = temp_dir.path().join("decompress");
        decompress_file_with_password(dest, &decompress_dest, "rust".into())
            .await
            .expect("decompress ok");
        assert!(decompress_dest.exists());
        let decompress_file = decompress_dest.join("file1.txt");
        assert!(decompress_file.exists());

        assert_eq!(
            async_fs::read_to_string(&decompress_file).await.unwrap(),
            content
        );
    }
}

#[cfg(feature = "compress")]
async fn test_compression_method(methods: &[EncoderConfiguration]) {
    let content = async_fs::read("tests/resources/decompress_x86.exe")
        .await
        .unwrap();

    let bytes: Vec<u8>;

    {
        let mut writer = ArchiveWriter::new(futures_lite::io::Cursor::new(Vec::<u8>::new()))
            .await
            .unwrap();
        let file = ArchiveEntry::new_file("data/decompress_x86.exe");
        let directory = ArchiveEntry::new_directory("data");

        writer.set_content_methods(methods.to_vec());
        writer
            .push_archive_entry(file, Some(content.as_slice()))
            .await
            .unwrap();
        writer
            .push_archive_entry::<&[u8]>(directory, None)
            .await
            .unwrap();
        let cursor = writer.finish().await.unwrap();
        bytes = cursor.into_inner();
    }

    let mut reader = ArchiveReader::open_from_bytes(bytes, Password::empty())
        .await
        .unwrap();

    assert_eq!(reader.archive().files.len(), 2);

    reader
        .archive()
        .files
        .iter()
        .filter(|file| !file.is_directory)
        .for_each(|file| {
            let mut file_methods = Vec::<EncoderMethod>::new();
            reader
                .file_compression_methods(file.name(), &mut file_methods)
                .expect("can't read compression method");

            for (file_method, method) in file_methods.iter().zip(methods) {
                assert_eq!(file_method.name(), method.method.name());
            }
        });

    assert!(
        reader
            .archive()
            .files
            .iter()
            .any(|file| file.name() == "data")
    );
    assert!(
        reader
            .archive()
            .files
            .iter()
            .any(|file| file.name() == "data/decompress_x86.exe")
    );

    let data = reader.read_file("data/decompress_x86.exe").await.unwrap();

    fn hash(data: &[u8]) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }

    assert_eq!(hash(&content), hash(&data));
}

#[cfg(feature = "compress")]
#[tokio::test]
async fn compress_with_copy_algorithm() {
    test_compression_method(&[EncoderMethod::COPY.into()]).await;
}

#[cfg(feature = "compress")]
#[tokio::test]
async fn compress_with_delta_lzma_algorithm() {
    for i in 1..=4 {
        test_compression_method(&[
            EncoderMethod::LZMA.into(),
            DeltaOptions::from_distance(i).into(),
        ])
        .await;
    }
}

#[cfg(feature = "compress")]
#[tokio::test]
async fn compress_with_delta_lzma2_algorithm() {
    for i in 1..=4 {
        test_compression_method(&[
            EncoderMethod::LZMA2.into(),
            DeltaOptions::from_distance(i).into(),
        ])
        .await;
    }
}

#[cfg(feature = "compress")]
#[tokio::test]
async fn compress_with_bcj_x86_lzma2_algorithm() {
    test_compression_method(&[
        EncoderMethod::LZMA2.into(),
        EncoderMethod::BCJ_X86_FILTER.into(),
    ])
    .await;
}

#[cfg(feature = "compress")]
#[tokio::test]
async fn compress_with_bcj_arm_lzma2_algorithm() {
    test_compression_method(&[
        EncoderMethod::LZMA2.into(),
        EncoderMethod::BCJ_ARM_FILTER.into(),
    ])
    .await;
}

#[cfg(feature = "compress")]
#[tokio::test]
async fn compress_with_bcj_arm64_lzma2_algorithm() {
    test_compression_method(&[
        EncoderMethod::LZMA2.into(),
        EncoderMethod::BCJ_ARM64_FILTER.into(),
    ])
    .await;
}

#[cfg(feature = "compress")]
#[tokio::test]
async fn compress_with_bcj_arm_thumb_lzma2_algorithm() {
    test_compression_method(&[
        EncoderMethod::LZMA2.into(),
        EncoderMethod::BCJ_ARM_THUMB_FILTER.into(),
    ])
    .await;
}

#[cfg(feature = "compress")]
#[tokio::test]
async fn compress_with_bcj_ia64_lzma2_algorithm() {
    test_compression_method(&[
        EncoderMethod::LZMA2.into(),
        EncoderMethod::BCJ_IA64_FILTER.into(),
    ])
    .await;
}

#[cfg(feature = "compress")]
#[tokio::test]
async fn compress_with_bcj_sparc_lzma2_algorithm() {
    test_compression_method(&[
        EncoderMethod::LZMA2.into(),
        EncoderMethod::BCJ_SPARC_FILTER.into(),
    ])
    .await;
}

#[cfg(feature = "compress")]
#[tokio::test]
async fn compress_with_bcj_ppc_lzma2_algorithm() {
    test_compression_method(&[
        EncoderMethod::LZMA2.into(),
        EncoderMethod::BCJ_PPC_FILTER.into(),
    ])
    .await;
}

#[cfg(feature = "compress")]
#[tokio::test]
async fn compress_with_bcj_riscv_lzma2_algorithm() {
    test_compression_method(&[
        EncoderMethod::LZMA2.into(),
        EncoderMethod::BCJ_RISCV_FILTER.into(),
    ])
    .await;
}

#[cfg(feature = "compress")]
#[tokio::test]
async fn compress_with_lzma_algorithm() {
    test_compression_method(&[EncoderMethod::LZMA.into()]).await;
}

#[cfg(feature = "compress")]
#[tokio::test]
async fn compress_with_lzma2_algorithm() {
    test_compression_method(&[EncoderMethod::LZMA2.into()]).await;
}

#[cfg(all(feature = "compress", feature = "ppmd"))]
#[tokio::test]
async fn compress_with_ppmd_algorithm() {
    test_compression_method(&[EncoderMethod::PPMD.into()]).await;
}

#[cfg(all(feature = "compress", feature = "brotli"))]
#[tokio::test]
async fn compress_with_brotli_standard_algorithm() {
    test_compression_method(&[BrotliOptions::default().with_skippable_frame_size(0).into()]).await;
}

#[cfg(all(feature = "compress", feature = "brotli"))]
#[tokio::test]
async fn compress_with_brotli_skippable_algorithm() {
    test_compression_method(&[BrotliOptions::default()
        .with_skippable_frame_size(64 * 1024)
        .into()])
    .await;
}

#[cfg(all(feature = "compress", feature = "bzip2"))]
#[tokio::test]
async fn compress_with_bzip2_algorithm() {
    test_compression_method(&[EncoderMethod::BZIP2.into()]).await;
}

#[cfg(all(feature = "compress", feature = "deflate"))]
#[tokio::test]
async fn compress_with_deflate_algorithm() {
    test_compression_method(&[EncoderMethod::DEFLATE.into()]).await;
}

#[cfg(all(feature = "compress", feature = "lz4"))]
#[tokio::test]
async fn compress_with_lz4_algorithm() {
    test_compression_method(&[Lz4Options::default().with_skippable_frame_size(0).into()]).await;
}

#[cfg(all(feature = "compress", feature = "lz4"))]
#[tokio::test]
async fn compress_with_lz4_skippable_algorithm() {
    test_compression_method(&[Lz4Options::default()
        .with_skippable_frame_size(128 * 1024)
        .into()])
    .await;
}

#[cfg(all(feature = "compress", feature = "lz4"))]
#[tokio::test]
async fn compress_with_zstd_algorithm() {
    test_compression_method(&[EncoderMethod::ZSTD.into()]).await;
}

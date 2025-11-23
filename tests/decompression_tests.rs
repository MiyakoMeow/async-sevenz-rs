use futures_lite::StreamExt;
use futures_lite::io::{AsyncReadExt, Cursor};
use std::path::PathBuf;

use async_sevenz::decompress_file;
use async_sevenz::{Archive, ArchiveReader, BlockDecoder, Password};
use tempfile::tempdir;

#[tokio::test]
async fn decompress_single_empty_file_unencoded_header() {
    let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    source_file.push("tests/resources/single_empty_file.7z");
    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().to_path_buf();
    let mut file1_path = target.clone();
    file1_path.push("empty.txt");

    decompress_file(source_file, target).await.unwrap();

    assert_eq!(async_fs::read_to_string(file1_path).await.unwrap(), "");
}

#[tokio::test]
async fn decompress_two_empty_files_unencoded_header() {
    let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    source_file.push("tests/resources/two_empty_file.7z");
    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().to_path_buf();
    let mut file1_path = target.clone();
    file1_path.push("file1.txt");
    let mut file2_path = target.clone();
    file2_path.push("file2.txt");

    decompress_file(source_file, target).await.unwrap();

    assert_eq!(async_fs::read_to_string(file1_path).await.unwrap(), "");
    assert_eq!(async_fs::read_to_string(file2_path).await.unwrap(), "");
}

#[tokio::test]
async fn decompress_lzma_single_file_unencoded_header() {
    let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    source_file.push("tests/resources/single_file_with_content_lzma.7z");
    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().to_path_buf();
    let mut file1_path = target.clone();
    file1_path.push("file.txt");

    decompress_file(source_file, target).await.unwrap();

    assert_eq!(
        async_fs::read_to_string(file1_path).await.unwrap(),
        "this is a file\n"
    );
}

#[tokio::test]
async fn decompress_lzma2_bcj_x86_file() {
    let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    source_file.push("tests/resources/decompress_example_lzma2_bcj_x86.7z");
    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().to_path_buf();
    let mut file1_path = target.clone();
    file1_path.push("decompress.exe");

    decompress_file(source_file, target).await.unwrap();

    let mut expected_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    expected_file.push("tests/resources/decompress_x86.exe");

    assert_eq!(
        async_fs::read(file1_path).await.unwrap(),
        async_fs::read(expected_file).await.unwrap(),
        "decompressed files do not match!"
    );
}

#[tokio::test]
async fn decompress_bcj_arm64_file() {
    let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    source_file.push("tests/resources/decompress_example_bcj_arm64.7z");
    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().to_path_buf();
    let mut file1_path = target.clone();
    file1_path.push("decompress_arm64.exe");

    decompress_file(source_file, target).await.unwrap();

    let mut expected_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    expected_file.push("tests/resources/decompress_arm64.exe");

    assert_eq!(
        async_fs::read(file1_path).await.unwrap(),
        async_fs::read(expected_file).await.unwrap(),
        "decompressed files do not match!"
    );
}

#[tokio::test]
async fn decompress_lzma_multiple_files_encoded_header() {
    let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    source_file.push("tests/resources/two_files_with_content_lzma.7z");
    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().to_path_buf();
    let mut file1_path = target.clone();
    file1_path.push("file1.txt");
    let mut file2_path = target.clone();
    file2_path.push("file2.txt");

    decompress_file(source_file, target).await.unwrap();

    assert_eq!(
        async_fs::read_to_string(file1_path).await.unwrap(),
        "file one content\n"
    );
    assert_eq!(
        async_fs::read_to_string(file2_path).await.unwrap(),
        "file two content\n"
    );
}

#[tokio::test]
async fn decompress_delta_lzma_single_file_unencoded_header() {
    let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    source_file.push("tests/resources/delta.7z");
    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().to_path_buf();
    let mut file1_path = target.clone();
    file1_path.push("delta.txt");

    decompress_file(source_file, target).await.unwrap();

    assert_eq!(
        async_fs::read_to_string(file1_path).await.unwrap(),
        "aaaabbbbcccc"
    );
}

#[tokio::test]
async fn decompress_copy_lzma2_single_file() {
    let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    source_file.push("tests/resources/copy.7z");
    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().to_path_buf();
    let mut file1_path = target.clone();
    file1_path.push("copy.txt");

    decompress_file(source_file, target).await.unwrap();

    assert_eq!(
        async_fs::read_to_string(file1_path).await.unwrap(),
        "simple copy encoding"
    );
}

#[cfg(feature = "ppmd")]
#[tokio::test]
async fn decompress_ppmd_single_file() {
    let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    source_file.push("tests/resources/ppmd.7z");

    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().to_path_buf();
    let mut file1_path = target.clone();
    file1_path.push("apache2.txt");

    decompress_file(source_file, target).await.unwrap();
    let decompressed_content = async_fs::read_to_string(file1_path).await.unwrap();

    let expected = async_fs::read_to_string("tests/resources/apache2.txt")
        .await
        .unwrap();

    assert_eq!(decompressed_content, expected);
}

#[cfg(feature = "bzip2")]
#[tokio::test]
async fn decompress_bzip2_file() {
    let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    source_file.push("tests/resources/bzip2_file.7z");
    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().to_path_buf();

    let mut hello_path = target.clone();
    hello_path.push("hello.txt");

    let mut foo_path = target.clone();
    foo_path.push("foo.txt");

    decompress_file(source_file, target).await.unwrap();

    assert_eq!(
        async_fs::read_to_string(hello_path).await.unwrap(),
        "world\n"
    );
    assert_eq!(async_fs::read_to_string(foo_path).await.unwrap(), "bar\n");
}

/// zstdmt (which 7zip ZS uses), does encapsulate brotli data in a special frames,
/// for which we need to have custom logic to decode and encode to.
#[cfg(feature = "brotli")]
#[tokio::test]
async fn decompress_zstdmt_brotli_file() {
    let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    source_file.push("tests/resources/zstdmt-brotli.7z");

    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().to_path_buf();

    let mut license_path = target.clone();
    license_path.push("LICENSE");

    decompress_file(source_file, target).await.unwrap();

    assert!(
        async_fs::read_to_string(license_path)
            .await
            .unwrap()
            .contains("Apache License")
    );
}

#[cfg(feature = "lz4")]
#[tokio::test]
async fn decompress_zstdmt_lz4_file() {
    let mut source_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    source_file.push("tests/resources/zstdmt-lz4.7z");

    let temp_dir = tempdir().unwrap();
    let target = temp_dir.path().to_path_buf();

    let mut license_path = target.clone();
    license_path.push("LICENSE");

    decompress_file(source_file, target).await.unwrap();

    assert!(
        async_fs::read_to_string(license_path)
            .await
            .unwrap()
            .contains("Apache License")
    );
}

#[tokio::test]
async fn test_bcj2() {
    let archive = Archive::open_with_password(
        "tests/resources/7za433_7zip_lzma2_bcj2.7z",
        &Password::empty(),
    )
    .await
    .unwrap();
    for i in 0..archive.blocks.len() {
        let password = Password::empty();
        let data = async_fs::read("tests/resources/7za433_7zip_lzma2_bcj2.7z")
            .await
            .unwrap();
        let mut cursor = Cursor::new(data);
        let fd = BlockDecoder::new(1, i, &archive, &password, &mut cursor);
        println!("entry_count:{}", fd.entry_count());
        fd.for_each_entries(&mut |entry, reader| {
            println!("{}=>{:?}", entry.has_stream, entry.name());
            Box::pin(async move {
                let mut buf = Vec::new();
                AsyncReadExt::read_to_end(reader, &mut buf).await?;
                Ok(true)
            })
        })
        .await
        .unwrap();
    }
}

#[tokio::test]
async fn test_entry_compressed_size() {
    let mut dir = async_fs::read_dir("tests/resources").await.unwrap();
    while let Some(res) = dir.next().await {
        let path = res.unwrap().path();
        if path.to_string_lossy().ends_with("7z") {
            println!("{path:?}");
            let archive = Archive::open_with_password(&path, &Password::empty())
                .await
                .unwrap();
            for i in 0..archive.blocks.len() {
                let fi = archive.stream_map.block_first_file_index[i];
                let file = &archive.files[fi];
                println!(
                    "\t:{}\tsize={}, \tcompressed={}",
                    file.name(),
                    file.size,
                    file.compressed_size
                );
                if file.has_stream && file.size > 0 {
                    assert!(file.compressed_size > 0);
                }
            }
        }
    }
}

#[tokio::test]
async fn test_get_file_by_path() {
    // non_solid.7z and solid.7z are expected to have the same content.
    let mut non_solid_reader =
        ArchiveReader::open("tests/resources/non_solid.7z", Password::empty())
            .await
            .unwrap();
    let mut solid_reader = ArchiveReader::open("tests/resources/solid.7z", Password::empty())
        .await
        .unwrap();

    let paths: Vec<String> = non_solid_reader
        .archive()
        .files
        .iter()
        .filter(|file| !file.is_directory)
        .map(|file| file.name.clone())
        .collect();

    for path in paths.iter() {
        let data0 = non_solid_reader.read_file(path.as_str()).await.unwrap();
        let data1 = solid_reader.read_file(path.as_str()).await.unwrap();

        assert!(!data0.is_empty());
        assert!(!data1.is_empty());
        assert_eq!(&data0, &data1);
    }
}

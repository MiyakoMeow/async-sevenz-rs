#[cfg(feature = "compress")]
use async_sevenz::*;
#[cfg(feature = "compress")]
use tempfile::*;
#[cfg(feature = "compress")]
#[tokio::test]
async fn compress_multi_files_solid() {
    use futures_lite::io::Cursor;

    let temp_dir = tempdir().unwrap();
    let folder = temp_dir.path().join("folder");
    async_fs::create_dir(&folder).await.unwrap();
    let mut files = Vec::with_capacity(100);
    let mut contents = Vec::with_capacity(100);
    for i in 1..=10000 {
        let name = format!("file{i}.txt");
        let content = format!("file{i} with content");
        async_fs::write(folder.join(&name), &content).await.unwrap();
        files.push(name);
        contents.push(content);
    }
    let dest = temp_dir.path().join("folder.7z");

    let mut sz = ArchiveWriter::new(Cursor::new(Vec::<u8>::new()))
        .await
        .unwrap();
    sz.push_source_path(&folder, |_| async { true })
        .await
        .unwrap();
    let cursor = sz.finish().await.expect("compress ok");
    let data = cursor.into_inner();
    async_fs::write(&dest, data).await.unwrap();

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
async fn compress_multi_files_mix_solid_and_non_solid() {
    use futures_lite::io::Cursor;

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

    let mut sz = ArchiveWriter::new(Cursor::new(Vec::<u8>::new()))
        .await
        .unwrap();

    // solid compression
    sz.push_source_path(&folder, |_| async { true })
        .await
        .unwrap();

    // non solid compression
    for i in 101..=200 {
        let name = format!("file{i}.txt");
        let content = format!("file{i} with content");
        async_fs::write(folder.join(&name), &content).await.unwrap();
        files.push(name.clone());
        contents.push(content);

        let src = folder.join(&name);
        let data = async_fs::read(&src).await.unwrap();
        sz.push_archive_entry(
            ArchiveEntry::from_path(&src, name).await,
            Some(Cursor::new(data)),
        )
        .await
        .expect("ok");
    }

    let cursor = sz.finish().await.expect("compress ok");
    let data = cursor.into_inner();
    async_fs::write(&dest, data).await.unwrap();

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

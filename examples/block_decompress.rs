use async_fs as afs;
use std::path::PathBuf;

use async_sevenz::{Archive, BlockDecoder, Password};

#[tokio::main]
async fn main() {
    let password = Password::empty();
    let archive = Archive::open_with_password("examples/data/sample.7z", &password)
        .await
        .unwrap();
    let data = afs::read("examples/data/sample.7z").await.unwrap();
    let mut cursor = futures::io::Cursor::new(data);
    let block_count = archive.blocks.len();
    let my_file_name = "7zFormat.txt";

    for block_index in 0..block_count {
        let forder_dec = BlockDecoder::new(1, block_index, &archive, &password, &mut cursor);

        if !forder_dec
            .entries()
            .iter()
            .any(|entry| entry.name() == my_file_name)
        {
            // skip the folder if it does not contain the file we want
            continue;
        }
        let dest = PathBuf::from("examples/data/sample_mt/");

        forder_dec
            .for_each_entries(&mut |entry, reader| {
                if entry.name() == my_file_name {
                    let dest = dest.join(entry.name());
                    Box::pin(async move {
                        async_sevenz::default_entry_extract_fn(entry, reader, &dest).await?;
                        Ok(true)
                    })
                } else {
                    Box::pin(async move {
                        let mut buf = Vec::new();
                        futures::io::AsyncReadExt::read_to_end(reader, &mut buf).await?;
                        Ok(true)
                    })
                }
            })
            .await
            .expect("ok");
    }
}

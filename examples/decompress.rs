use std::time::Instant;

use async_fs as afs;
use async_sevenz::default_entry_extract_fn;
use futures::io::Cursor;

fn main() {
    let instant = Instant::now();
    smol::block_on(async {
        let data = afs::read("examples/data/sample.7z").await.unwrap();
        async_sevenz::decompress_with_extract_fn_and_password(
            Cursor::new(data),
            "examples/data/sample",
            "pass".into(),
            |entry, reader, dest| {
                Box::pin(async move {
                    println!("start extract {}", entry.name());
                    let r = default_entry_extract_fn(entry, reader, dest).await;
                    println!("complete extract {}", entry.name());
                    r
                })
            },
        )
        .await
    })
    .expect("complete");
    println!("decompress done:{:?}", instant.elapsed());
}

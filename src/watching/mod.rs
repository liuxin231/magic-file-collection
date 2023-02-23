use crate::watching::offset::{get_offset_by_key, set_offset};
use futures::channel::mpsc::{channel, Receiver};
use futures::{SinkExt, StreamExt};
use notify::event::ModifyKind;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::io::BufRead;
use std::os::unix::fs::FileExt;
use std::path::Path;

pub mod offset;

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);
    let watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        Config::default(),
    )?;
    Ok((watcher, rx))
}

pub async fn async_watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;
    while let Some(Ok(res)) = rx.next().await {
        if let EventKind::Modify(ModifyKind::Data(_)) = res.kind {
            let paths = res.paths;
            for path in paths {
                let path = path.to_str().unwrap().to_string();
                read_file_offset_content(path);
            }
        }
    }
    Ok(())
}

fn read_file_offset_content(path: String) {
    let file = std::fs::OpenOptions::new().read(true).open(&path).unwrap();
    let file_metadata = file.metadata().unwrap();
    let file_size = file_metadata.len() as usize;
    let offset = get_offset_by_key(&path);
    if offset >= file_size {
        return;
    }
    let read_size = file_size as usize - offset as usize;
    let mut read_buf = vec![0u8; read_size];
    file.read_at(&mut read_buf, offset as u64).unwrap();
    set_offset(&path, file_size);
    let mut cursor = std::io::Cursor::new(&read_buf).lines();
    while let Some(line) = cursor.next() {
        let line = line.unwrap();
        if line.is_empty() {
            continue;
        }
        tracing::info!("content: {}", line);
    }
}

use crate::watching::offset::{get_offset_by_key, set_offset};
use futures::channel::mpsc::{channel, Receiver};
use futures::{SinkExt, StreamExt};
use notify::event::ModifyKind;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::io::BufRead;
#[cfg(unix)]
use std::os::unix::fs::FileExt;
#[cfg(windows)]
use std::os::windows::fs::FileExt;
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

pub async fn async_watch<P: AsRef<Path>>(
    paths: Vec<P>,
    file_name_regex: Vec<String>,
    message_sender: tokio::sync::mpsc::Sender<String>,
) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;
    for path in paths {
        while let Err(error) = watcher.watch(path.as_ref(), RecursiveMode::Recursive) {
            tracing::info!(
                "watch file: {:?}, error: {}",
                path.as_ref(),
                error.to_string()
            );
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
    while let Some(Ok(res)) = rx.next().await {
        if let EventKind::Modify(ModifyKind::Data(_)) = res.kind {
            let paths = res.paths;
            for path in paths {
                if path.is_dir() {
                    continue;
                }
                let file_name = path.file_name().unwrap().to_str().unwrap();
                let is_match_file_name = file_name_regex
                    .iter()
                    .map(|item| regex::Regex::new(&item).unwrap().is_match(&file_name))
                    .collect::<Vec<bool>>()
                    .contains(&true);
                if is_match_file_name {
                    let path = path.to_str().unwrap().to_string();
                    read_file_offset_content(path, message_sender.clone()).await;
                }
            }
        }
    }
    Ok(())
}

async fn read_file_offset_content(path: String, message_sender: tokio::sync::mpsc::Sender<String>) {
    let file = match std::fs::OpenOptions::new().read(true).open(&path) {
        Ok(file) => file,
        Err(_) => {
            return;
        }
    };
    let file_metadata = file.metadata().unwrap();
    let file_size = file_metadata.len() as usize;
    let offset = get_offset_by_key(&path);
    if offset >= file_size {
        set_offset(&path, 0, false);
        return;
    }
    let read_size = file_size as usize - offset as usize;
    let mut read_buf = vec![0u8; read_size];
    #[cfg(windows)]
    file.seek_read(&mut read_buf, offset as u64).unwrap();
    #[cfg(unix)]
    file.read_at(&mut read_buf, offset as u64).unwrap();
    set_offset(&path, file_size, true);
    let mut cursor = std::io::Cursor::new(&read_buf).lines();
    while let Some(line) = cursor.next() {
        let line = line.unwrap();
        if line.is_empty() {
            continue;
        }
        message_sender.send(line).await.unwrap();
    }
}

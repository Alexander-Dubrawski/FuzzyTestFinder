use crate::{FztError, RunnerConfig, SearchEngine};
use notify::{
    Event, EventKind, RecursiveMode, Result as NotifyResult, Watcher,
    event::{DataChange, ModifyKind},
};

use std::{path::Path, sync::mpsc};

pub fn watch<SE: SearchEngine + Clone + Send>(config: RunnerConfig<SE>) -> Result<(), FztError> {
    let (notify_tx, notify_rx) = mpsc::channel::<NotifyResult<Event>>();
    let mut watcher = notify::recommended_watcher(notify_tx)?;
    let runner = config.clone().into_runner()?;
    watcher.watch(Path::new(runner.root_path()), RecursiveMode::Recursive)?;

    let file_ext = match config.language {
        crate::Language::Rust => "rs",
        crate::Language::Python { .. } => "py",
        crate::Language::Java { .. } => "java",
    };

    // Get first selection
    let mut init_run = true;

    loop {
        let (tx, rx) = mpsc::channel::<String>();
        let mut local_config = config.clone();
        if !init_run {
            local_config.mode = crate::RunnerMode::Last;
            local_config.update_history = false;
        } else {
            if local_config.mode != crate::RunnerMode::All {
                init_run = false;
            }
        }
        let handle = std::thread::spawn(move || -> Result<(), FztError> {
            local_config.into_runner()?.run(Some(rx))
        });
        let event = loop {
            let event = notify_rx.recv()??;
            let is_rust_file = event
                .paths
                .iter()
                .any(|p| p.extension().map_or(false, |ext| ext == file_ext));
            let is_content_modify = matches!(
                event.kind,
                EventKind::Modify(ModifyKind::Data(DataChange::Content))
            );
            if is_rust_file && is_content_modify {
                break event;
            }
        };

        println!("\nFile change detected: {:?}\n", event);
        println!("\nTry stopping currently running tests\n");
        let _ = tx.send(String::from("file change"));
        handle.join().unwrap()?;
    }
}

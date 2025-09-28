use crate::{FztError, RunnerConfig, SearchEngine};
use notify::{Event, RecursiveMode, Result as NotifyResult, Watcher};

use std::{path::Path, sync::mpsc};

pub fn watch<SE: SearchEngine + Clone + Send>(config: RunnerConfig<SE>) -> Result<(), FztError> {
    let (notify_tx, notify_rx) = mpsc::channel::<NotifyResult<Event>>();
    let mut watcher = notify::recommended_watcher(notify_tx)?;
    let runner = config.clone().into_runner()?;
    watcher.watch(Path::new(runner.root_path()), RecursiveMode::Recursive)?;

    loop {
        let (tx, rx) = mpsc::channel::<String>();
        let local_config = config.clone();
        std::thread::spawn(move || {
            local_config.into_runner().unwrap().run(Some(rx)).unwrap();
        });
        println!("Watching for file changes.");
        let event = notify_rx.recv()??;
        println!("File change detected: {:?}", event);
        let _ = tx.send(String::from("file change"));
    }
}

use std::{path::PathBuf, str::FromStr, time::Duration};

use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;

pub struct Watchdog {
    root: String,
}

impl Watchdog {
    pub fn start<F>(&self, f: F)
    where
        F: Fn(),
    {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut debouncer = new_debouncer(Duration::from_secs(1), None, tx).unwrap();

        debouncer
            .watcher()
            .watch(
                &PathBuf::from_str(&self.root).unwrap(),
                RecursiveMode::Recursive,
            )
            .unwrap();

        for events in rx {
            // while let Ok(ref event_list) = events {

            for event_list in events {
                let filtered = event_list
                    .iter()
                    .filter(|e| {
                        let prefix = e.path.strip_prefix(&self.root).unwrap();
                        !(prefix.starts_with("target") || prefix.starts_with(".git"))
                    })
                    .collect::<Vec<_>>();
                if !filtered.is_empty() {
                    dbg!(filtered.iter().map(|ref x| &x.path).collect::<Vec<_>>());
                    f();
                }
            }
        }
    }
}

impl Watchdog {
    pub fn new(root: String) -> Self {
        Self { root }
    }
}

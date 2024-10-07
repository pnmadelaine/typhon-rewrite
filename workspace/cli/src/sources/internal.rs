use std::{collections::BTreeMap, fs, io::Read, path::Path};

use serde::{Deserialize, Serialize};
use typhon_nix::*;

use crate::runtime::run;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Entry {
    source: Source,
    lock: Lock,
    out_path: String,
}

fn lock(source: &Source) -> Result<(Lock, String), Error> {
    run(async {
        let mut store = NixStore::new().await?;
        store.lock(source).await
    })
}

impl Entry {
    fn new(source: Source) -> Self {
        let (lock, out_path) = lock(&source).unwrap();
        Self {
            source,
            lock,
            out_path,
        }
    }

    fn update(&mut self) {
        let (lock, out_path) = lock(&self.source).unwrap();
        self.lock = lock;
        self.out_path = out_path;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Sources {
    #[serde(flatten)]
    entries: BTreeMap<String, Entry>,
}

impl Sources {
    pub(crate) fn add(&mut self, name: String, source: Source) {
        self.entries.insert(name, Entry::new(source));
    }

    pub(crate) fn remove(&mut self, name: &String) {
        self.entries.remove(name);
    }

    pub(crate) fn open(path: &Path) -> Self {
        let mut file = fs::File::open(path).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        serde_json::from_str(&content).unwrap()
    }

    pub(crate) fn update(&mut self, name: &str) {
        self.entries.get_mut(name).unwrap().update();
    }

    pub(crate) fn update_all(&mut self) {
        let () = self.entries.values_mut().map(Entry::update).collect::<()>();
    }

    pub(crate) fn write(&self, path: &Path) {
        fs::write(path, serde_json::to_string_pretty(self).unwrap()).unwrap();
    }
}

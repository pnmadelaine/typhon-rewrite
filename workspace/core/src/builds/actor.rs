use std::collections::{hash_map::Entry, BTreeSet, VecDeque};

use typhon_nix::RecursiveDerivation;

use super::{queue::*, *};
use crate::prelude::*;

type RootId = u128;

#[derive(Clone)]
pub(crate) struct Root {
    id: RootId,
    recv: watch::Receiver<()>,
}

pub(super) enum Cmd {
    Build {
        drv: String,
        priority: Priority,
        ret: Ret<Watch>,
    },
    Info {
        id: Id,
        ret: Ret<Option<Info>>,
    },
    Stderr {
        id: Id,
        ret: Ret<Result<Read, Error>>,
    },
    // ...
    BuildInner {
        drv: String,
        priority: Priority,
        recursive_derivation: RecursiveDerivation,
        missing: Vec<String>,
        ret: Ret<Watch>,
    },
}

struct Handle {
    started_at: SystemTime,
    stderr: Subscribe,
    abort: Option<oneshot::Sender<()>>,
}

struct Build {
    drv: String,
    priority: Priority,
    dependencies: HashSet<Id>,
    dependents: BTreeSet<(Priority, Id)>,
    roots: BTreeSet<(Priority, RootId)>,
}

pub struct WatchBuild {}

pub(super) struct St {
    scope: Scope,
    pool: Pool,
    count_builds: Id,
    count_roots: RootId,
    drvs: HashMap<String, Id>,
    builds: HashMap<Id, Build>,
    queue: Queue<Id, Priority>,
    roots: HashMap<RootId, Priority>,
    running: HashMap<Id, Handle>,
    // Settings
    max_running: usize,
}

impl St {
    pub(super) async fn new(scope: Scope, pool: Pool) -> Self {
        let max_running = 1;
        Self {
            scope,
            pool,
            count_builds: 0,
            count_roots: 0,
            drvs: HashMap::new(),
            builds: HashMap::new(),
            queue: Queue::new(),
            roots: HashMap::new(),
            running: HashMap::with_capacity(max_running),
            max_running,
        }
    }

    // ...

    fn build_inner(
        &mut self,
        drv: String,
        priority: Priority,
        recursive_derivation: RecursiveDerivation,
        missing: Vec<String>,
        ret: Ret<Watch>,
    ) {
        let root_id = self.count_roots;
        self.count_roots += 1;

        let missing: HashSet<String> = missing.into_iter().collect();
        if !missing.contains(&drv) {
            ret.send(Watch(Vec::new()));
            return;
        }

        let RecursiveDerivation(mut m) = recursive_derivation;
        let mut visited = BTreeMap::new();

        let mut insert_queue = VecDeque::new();
        let mut update_queue = VecDeque::new();
        let mut visit_queue = VecDeque::new();

        match self.drvs.entry(drv.to_owned()) {
            Entry::Occupied(entry) => {
                let u = *entry.get();
                let build = self.builds.get_mut(&u).unwrap();
                if build.priority < priority {
                    visited.insert(u, WatchBuild {});
                    for v in &build.dependencies {
                        update_queue.push_back((build.priority, u, *v));
                    }
                    build.priority = priority;
                } else {
                    visit_queue.push_back(u);
                }
                u
            }
            Entry::Vacant(entry) => {
                let u = self.count_builds;
                entry.insert(u);
                self.count_builds += 1;
                insert_queue.push_back((
                    u,
                    Build {
                        drv,
                        priority,
                        dependencies: HashSet::new(),
                        dependents: BTreeSet::new(),
                        roots: BTreeSet::from([(priority, root_id)]),
                    },
                ));
                u
            }
        };

        while let Some((u, mut build)) = insert_queue.pop_front() {
            visited.insert(u, WatchBuild {});
            let dependencies = m.remove(&build.drv).unwrap().dependencies;
            for drv in dependencies.into_keys() {
                if !missing.contains(&drv) {
                    continue;
                }
                let v = match self.drvs.entry(drv.clone()) {
                    Entry::Occupied(entry) => {
                        let v = *entry.get();
                        let build = self.builds.get_mut(&v).unwrap();
                        build.dependents.insert((priority, v));
                        if build.priority < priority {
                            visited.insert(v, WatchBuild {});
                            for w in &build.dependencies {
                                update_queue.push_back((build.priority, v, *w));
                            }
                            build.priority = priority;
                        } else {
                            visit_queue.push_back(v);
                        }
                        v
                    }
                    Entry::Vacant(entry) => {
                        let v = self.count_builds;
                        entry.insert(v);
                        self.count_builds += 1;
                        insert_queue.push_back((
                            v,
                            Build {
                                drv,
                                priority,
                                dependencies: HashSet::new(),
                                dependents: BTreeSet::from([(priority, u)]),
                                roots: BTreeSet::new(),
                            },
                        ));
                        v
                    }
                };
                build.dependencies.insert(v);
            }
            if build.dependencies.is_empty() {
                self.queue.push(u, priority);
            }
            self.builds.insert(u, build);
        }

        while let Some((old_priority, u, v)) = update_queue.pop_front() {
            let build = self.builds.get_mut(&v).unwrap();
            build.dependents.remove(&(old_priority, u));
            build.dependents.insert((priority, u));
            if build.priority < priority {
                visited.insert(u, WatchBuild {});
                for w in &build.dependencies {
                    update_queue.push_back((build.priority, v, *w));
                }
                build.priority = priority;
            } else {
                visit_queue.push_back(v);
            }
        }

        while let Some(u) = visit_queue.pop_front() {
            visited.entry(u).or_insert_with(|| {
                let build = self.builds.get(&u).unwrap();
                for v in &build.dependencies {
                    visit_queue.push_back(*v);
                }
                WatchBuild {}
            });
        }

        ret.send(Watch(visited.into_iter().map(|(_, watch)| watch).collect()));
    }

    // ...

    fn build(&mut self, drv: String, priority: Priority, ret: Ret<Watch>, addr: WeakAddr<Cmd>) {
        self.scope.spawn(async move {
            let mut store = nix::NixStore::new().await.unwrap();
            let recursive_derivation = nix::derivation(&drv).await.unwrap();
            let missing = store.missing(&drv).await.unwrap();
            addr.send(Cmd::BuildInner {
                drv,
                priority,
                recursive_derivation,
                missing,
                ret,
            })
            .await;
        });
    }

    fn stderr(&mut self, id: Id, ret: Ret<Result<Read, Error>>) {
        let maybe_subscribe = self.running.get(&id).map(|handle| &handle.stderr).cloned();
        let pool = self.pool.clone();
        self.scope.spawn(async move {
            if let Some(subscribe) = maybe_subscribe {
                ret.send(Ok(subscribe.read().await.unwrap()));
                return;
            }
            let mut conn = pool.get().await;
            let build = schema::builds::table
                .find(id)
                .first::<models::Build>(&mut conn)
                .await
                .optional()
                .unwrap();
            if build.is_none() {
                ret.send(Err(Error::BuildNotFound(id)));
                return;
            }
            let subscribe = Subscribe::new(format!("logs/builds/{id}.stderr"));
            ret.send(
                subscribe
                    .read()
                    .await
                    .map_err(|_| Error::StderrNotFound(id)),
            );
        });
    }
}

impl Exec for Cmd {
    type St = St;

    fn exec(self, state: &mut Self::St, addr: &WeakAddr<Self>) {
        match self {
            Cmd::Build { drv, priority, ret } => state.build(drv, priority, ret, addr.clone()),
            Cmd::Info { .. } => (),
            Cmd::Stderr { .. } => (),
            // ...
            Cmd::BuildInner {
                drv,
                priority,
                recursive_derivation,
                missing,
                ret,
            } => state.build_inner(drv, priority, recursive_derivation, missing, ret),
        }
    }

    async fn finish(state: Self::St) {
        let St { scope, .. } = state;
        scope.wait().await;
    }
}

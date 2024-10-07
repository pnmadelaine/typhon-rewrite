use super::{Error, *};
use crate::prelude::*;

pub(super) enum Cmd {
    Disable {
        ret: Res<(), Error>,
    },
    EditSecret {
        key: String,
        value: Option<String>,
        ret: Ret<()>,
    },
    Enable {
        ret: Res<(), Error>,
    },
    IsEnabled {
        ret: Ret<bool>,
    },
    Lock {
        ret: Res<(), Error>,
    },
    Secrets {
        ret: Ret<Vec<String>>,
    },
    SetPath {
        path: String,
        ret: Res<(), Error>,
    },
    SetSource {
        source: Source,
        ret: Res<(), Error>,
    },
    // ...
    Disabled,
    Enabled {
        inner: StInner,
    },
    LockResult {
        res: Result<(Lock, String), nix::Error>,
    },
}

pub(super) struct StInner {
    // jobsets: BTreeMap<String, Addr<jobsets::Cmd>>,
}

pub(super) struct St {
    scope: Scope,
    pool: Pool,
    // ...
    id: i32,
    inner: Option<StInner>,
    disabled: bool,
    // locking
    lock_handle: Option<AbortHandle>,
    lock_err: Option<String>,
    lock: Option<Lock>,
}

impl St {
    fn disable(&mut self, ret: Res<(), Error>, addr: WeakAddr<Cmd>) {
        ret.send(if self.inner.is_none() {
            Err(Error::ProjectIsNotEnabled)
        } else {
            let inner = self.inner.take();
            self.scope.spawn(async move {
                drop(inner);
                // TODO: wait for childs
                addr.send(Cmd::Disabled).await;
            });
            Ok(())
        });
    }

    fn edit_secret(&mut self, key: String, value: Option<String>, ret: Ret<()>) {
        let project_id = self.id.clone();
        let pool = self.pool.clone();
        self.scope.spawn(async move {
            let mut conn = pool.get().await;
            match value {
                Some(value) => diesel::insert_into(schema::secrets::table)
                    .values(models::NewSecret {
                        project_id,
                        key: &key,
                        value: &value,
                    })
                    .on_conflict((schema::secrets::project_id, schema::secrets::key))
                    .do_update()
                    .set(schema::secrets::value.eq(&value))
                    .execute(&mut conn)
                    .await
                    .map(|_| ())
                    .unwrap(),
                None => diesel::delete(
                    schema::secrets::table
                        .filter(schema::secrets::project_id.eq(project_id))
                        .filter(schema::secrets::key.eq(&key)),
                )
                .execute(&mut conn)
                .await
                .map(|_| ())
                .unwrap(),
            }
            ret.send(());
        });
    }

    fn enable(&mut self, ret: Res<(), Error>, addr: WeakAddr<Cmd>) {
        ret.send(if !self.disabled {
            Err(Error::ProjectIsNotDisabled)
        } else {
            self.disabled = false;
            self.scope.spawn(async move {
                // TODO
                let inner = StInner {};
                addr.send(Cmd::Enabled { inner }).await;
            });
            Ok(())
        });
    }

    fn is_enabled(&mut self, ret: Ret<bool>) {
        ret.send(self.inner.is_some());
    }

    fn lock(&mut self, ret: Res<(), Error>, addr: WeakAddr<Cmd>) {
        self.lock_handle.take().map(|handle| handle.abort());
        let project_id = self.id.clone();
        let pool = self.pool.clone();
        let _ = self.lock_handle.insert(self.scope.spawn(async move {
            let mut conn = pool.get().await;
            let (source, path): (Option<String>, Option<String>) = schema::projects::table
                .find(project_id)
                .select((schema::projects::source, schema::projects::path))
                .first(&mut conn)
                .await
                .unwrap();
            let (source, path) = match source {
                Some(source) => (
                    serde_json::from_str(&source).unwrap(),
                    path.as_deref().unwrap_or(""),
                ),
                None => {
                    ret.err(Error::SourceNotSet);
                    return;
                }
            };
            ret.ok(());
            let mut store = nix::NixStore::new().await.unwrap();

            // 1. lock
            let res = store.lock(&source).await;
            let (lock, mut out_path) = match res {
                Ok(x) => x,
                Err(err) => {
                    addr.send(Cmd::LockResult { res: Err(err) }).await;
                    return;
                }
            };
            out_path.push_str(path);

            // 2. evaluate
            let expr = lock.expr();
            let expr = format!("let x = {expr}; in import \"${{x}}/{path}\"");
            let res = nix::eval(&expr).await;
            let drv: String = match res {
                Ok(x) => x,
                Err(err) => {
                    addr.send(Cmd::LockResult { res: Err(err) }).await;
                    return;
                }
            };

            // 3. build
            let mut f = tokio::fs::File::open("/dev/null").await.unwrap();
            let res = store.build(&drv, &mut f).await;
            if let Err(err) = res {
                addr.send(Cmd::LockResult { res: Err(err) }).await;
                return;
            };

            let res = Ok((lock, out_path)); // WRONG OUT PATH
            addr.send(Cmd::LockResult { res }).await;
        }));
    }

    fn secrets(&mut self, ret: Ret<Vec<String>>) {
        let project_id = self.id.clone();
        let pool = self.pool.clone();
        self.scope.spawn(async move {
            let mut conn = pool.get().await;
            let res = schema::secrets::table
                .filter(schema::secrets::project_id.eq(project_id))
                .select(schema::secrets::key)
                .load(&mut conn)
                .await
                .unwrap();
            ret.send(res);
        });
    }

    fn set_path(&mut self, path: String, ret: Ret<Result<(), Error>>) {
        let project_id = self.id.clone();
        let pool = self.pool.clone();
        self.scope.spawn(async move {
            let mut conn = pool.get().await;
            diesel::update(schema::projects::table.find(project_id))
                .set(schema::projects::path.eq(path))
                .execute(&mut conn)
                .await
                .unwrap();
        });
        ret.send(Ok(()));
    }

    fn set_source(&mut self, source: Source, ret: Ret<Result<(), Error>>) {
        let project_id = self.id.clone();
        let pool = self.pool.clone();
        self.scope.spawn(async move {
            let mut conn = pool.get().await;
            let source = nix::json(&source).unwrap();
            diesel::update(schema::projects::table.find(project_id))
                .set(schema::projects::source.eq(source))
                .execute(&mut conn)
                .await
                .unwrap();
        });
        ret.send(Ok(()));
    }

    // ...

    fn disabled(&mut self) {
        self.disabled = true;
    }

    fn enabled(&mut self, inner: StInner) {
        self.inner = Some(inner);
        self.disabled = false;
    }

    fn lock_result(&mut self, res: Result<(Lock, String), nix::Error>) {
        self.lock_handle.take();
        let (lock, out_path) = match res {
            Ok(x) => x,
            Err(err) => {
                let _ = self.lock_err.insert(format!("{err:?}"));
                return;
            }
        };
        self.lock_err.take();
        let project_id = self.id.clone();
        let pool = self.pool.clone();
        let created_at = SystemTime::now();
        self.scope.spawn(async move {
            let mut conn = pool.get().await;
            let lock = &serde_json::to_string(&lock).unwrap();
            let out_path = &out_path;
            diesel::insert_into(schema::project_locks::table)
                .values(models::NewProjectLock {
                    project_id,
                    lock,
                    out_path,
                    created_at,
                })
                .execute(&mut conn)
                .await
                .unwrap();
        });
    }

    // ...

    pub(super) async fn new(scope: Scope, pool: Pool, name: String) -> Self {
        let mut conn = pool.get().await;
        let project: models::Project = schema::projects::table
            .filter(schema::projects::name.eq(&name))
            .first(&mut conn)
            .await
            .unwrap();
        drop(conn);
        Self {
            scope,
            pool,
            id: project.id,
            inner: None,
            disabled: true,
            lock_handle: None,
            lock_err: None,
            lock: None,
        }
    }
}

impl Exec for Cmd {
    type St = St;

    fn exec(self, state: &mut Self::St, send: &WeakAddr<Self>) {
        match self {
            Cmd::Disable { ret } => state.disable(ret, send.clone()),
            Cmd::EditSecret { key, value, ret } => state.edit_secret(key, value, ret),
            Cmd::Enable { ret } => state.enable(ret, send.clone()),
            Cmd::IsEnabled { ret } => state.is_enabled(ret),
            Cmd::Lock { ret } => state.lock(ret, send.clone()),
            Cmd::Secrets { ret } => state.secrets(ret),
            Cmd::SetPath { path, ret } => state.set_path(path, ret),
            Cmd::SetSource { source, ret } => state.set_source(source, ret),
            // ...
            Cmd::Disabled => state.disabled(),
            Cmd::Enabled { inner } => state.enabled(inner),
            Cmd::LockResult { res } => state.lock_result(res),
        }
    }

    async fn finish(_state: Self::St) {
        // TODO
    }
}

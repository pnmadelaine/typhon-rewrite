use super::{helpers::*, Error};
use crate::{prelude::*, projects};

pub(super) enum Cmd {
    DeleteProject {
        name: String,
        ret: Res<(), Error>,
    },
    NewProject {
        name: String,
        ret: Res<(), Error>,
    },
    Project {
        name: String,
        ret: Res<projects::Addr, Error>,
    },
    Projects {
        ret: Ret<Vec<String>>,
    },
    RenameProject {
        from: String,
        to: String,
        ret: Res<(), Error>,
    },
    // ...
    RemoveProject {
        name: String,
    },
}

pub(super) struct St {
    projects: BTreeMap<String, Option<projects::Addr>>,
    scope: Scope,
    pool: Pool,
}

impl St {
    fn delete_project(&mut self, name: String, ret: Res<(), Error>, addr: WeakAddr<Cmd>) {
        ret.do_send(|| {
            let project = self
                .projects
                .get_mut(&name)
                .ok_or_else(|| Error::ProjectNotFound(name.clone()))?;
            let project = project
                .take()
                .ok_or_else(|| Error::ProjectIsBeingDeleted(name.clone()))?;
            self.scope.spawn({
                let pool = self.pool.clone();
                async move {
                    project.wait().await;
                    let mut conn = pool.get().await;
                    diesel::delete(schema::projects::table)
                        .filter(schema::projects::name.eq(&name))
                        .execute(&mut conn)
                        .await
                        .unwrap();
                    addr.send(Cmd::RemoveProject { name }).await;
                }
            });
            Ok(())
        });
    }

    fn new_project(&mut self, name: String, ret: Res<(), Error>) {
        use std::collections::btree_map::Entry;
        if !validate_name(&name) {
            ret.err(Error::IllegalProjectName(name));
            return ();
        }
        let entry = match self.projects.entry(name.clone()) {
            Entry::Vacant(entry) => entry,
            Entry::Occupied(_) => {
                ret.err(Error::ProjectAlreadyExists(name));
                return ();
            }
        };
        entry.insert(Some(projects::new(
            &mut self.scope,
            self.pool.clone(),
            name,
        )));
        ret.ok(());
    }

    fn project(&self, name: String, ret: Ret<Result<projects::Addr, Error>>) {
        ret.send((|| {
            Ok(self
                .projects
                .get(&name)
                .ok_or_else(|| Error::ProjectNotFound(name.clone()))?
                .as_ref()
                .cloned()
                .ok_or_else(|| Error::ProjectIsBeingDeleted(name.clone()))?)
        })());
    }

    fn projects(&self, ret: Ret<Vec<String>>) {
        ret.send(self.projects.keys().cloned().collect());
    }

    fn rename_project(
        &mut self,
        from: String,
        to: String,
        ret: Res<(), Error>,
        addr: WeakAddr<Cmd>,
    ) {
        use std::collections::btree_map::Entry;
        ret.do_send(|| {
            let project = self
                .projects
                .get(&from)
                .ok_or_else(|| Error::ProjectNotFound(from.clone()))?
                .as_ref()
                .ok_or_else(|| Error::ProjectIsBeingDeleted(from.clone()))?
                .clone();
            if !validate_name(&to) {
                return Err(Error::IllegalProjectName(to));
            }
            let entry = match self.projects.entry(to.clone()) {
                Entry::Vacant(entry) => entry,
                Entry::Occupied(_) => return Err(Error::ProjectAlreadyExists(to)),
            };
            entry.insert(Some(project.clone()));
            self.projects.get_mut(&from).take();
            self.scope.spawn({
                let pool = self.pool.clone();
                async move {
                    let mut conn = pool.get().await;
                    diesel::update(schema::projects::table)
                        .filter(schema::projects::name.eq(&from))
                        .set(schema::projects::name.eq(&to))
                        .execute(&mut conn)
                        .await
                        .unwrap();
                    addr.send(Cmd::RemoveProject { name: from }).await;
                }
            });
            Ok(())
        });
    }

    // ...

    fn remove_project(&mut self, name: String) {
        self.projects.remove(&name);
    }

    // ...

    pub(super) async fn new(mut scope: Scope, pool: Pool) -> Self {
        let mut conn = pool.get().await;
        let projects = schema::projects::table
            .select(schema::projects::name)
            .load(&mut conn)
            .await
            .unwrap()
            .into_iter()
            .map(|name: String| {
                (
                    name.clone(),
                    Some(projects::load(&mut scope, pool.clone(), name)),
                )
            })
            .collect();
        drop(conn);
        Self {
            projects,
            scope,
            pool,
        }
    }
}

impl Exec for Cmd {
    type St = St;

    fn exec(self, state: &mut Self::St, addr: &WeakAddr<Self>) {
        match self {
            Self::DeleteProject { name, ret } => state.delete_project(name, ret, addr.clone()),
            Self::NewProject { name, ret } => state.new_project(name, ret),
            Self::Project { name, ret } => state.project(name, ret),
            Self::Projects { ret } => state.projects(ret),
            Self::RenameProject { from, to, ret } => {
                state.rename_project(from, to, ret, addr.clone())
            }
            // ...
            Cmd::RemoveProject { name } => state.remove_project(name),
        }
    }

    async fn finish(state: Self::St) {
        let St {
            projects,
            scope,
            pool: _,
        } = state;
        let projects = projects
            .into_iter()
            .map(|(_, addr)| addr)
            .flatten()
            .map(projects::Addr::wait);
        for p in projects {
            p.await;
        }
        scope.wait().await;
    }
}

use diesel_async::{
    pooled_connection::{bb8, AsyncDieselConnectionManager},
    AsyncPgConnection,
};

use crate::prelude::*;

type C = AsyncPgConnection;
type M = AsyncDieselConnectionManager<C>;

pub(crate) struct Conn<'a>(bb8::PooledConnection<'a, C>);

impl<'a> std::ops::Deref for Conn<'a> {
    type Target = bb8::PooledConnection<'a, C>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> std::ops::DerefMut for Conn<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Conn<'static> {
    pub(crate) async fn run_pending_migrations(&mut self) {
        schema::MIGRATIONS_ASYNC
            .run_pending_migrations(self)
            .await
            .unwrap();
    }
}

impl<'a> Conn<'a> {
    pub(crate) async fn repair(&mut self) {
        // TODO
    }
}

#[derive(Clone)]
pub(crate) struct Pool(bb8::Pool<C>);

impl Pool {
    pub(crate) async fn get(&self) -> Conn {
        Conn(self.0.get().await.unwrap())
    }

    pub(crate) async fn get_owned(&self) -> Conn<'static> {
        Conn(self.0.get_owned().await.unwrap())
    }

    pub(crate) fn new(database_url: &str) -> Self {
        let mgr = M::new(database_url);
        let pool = bb8::Pool::builder().build_unchecked(mgr);
        Self(pool)
    }
}

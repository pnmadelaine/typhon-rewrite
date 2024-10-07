use crate::prelude::*;

pub(crate) enum Cmd {}

pub(crate) struct St {}

impl St {
    // ...

    pub(crate) async fn new(_source: Source, _path: String) -> Self {
        Self {}
    }
}

impl Exec for Cmd {
    type St = St;

    fn exec(self, _state: &mut Self::St, _: &WeakAddr<Self>) {}

    async fn finish(_state: Self::St) {
        // TODO
    }
}

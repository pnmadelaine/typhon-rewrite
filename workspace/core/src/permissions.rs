#[derive(Clone)]
pub(crate) enum Permissions {}

impl Permissions {
    fn is_admin(&self) -> bool {
        todo!()
    }
    fn manages(&self, _project_name: String) -> bool {
        todo!()
    }
}

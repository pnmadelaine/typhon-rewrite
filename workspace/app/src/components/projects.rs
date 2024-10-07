use crate::prelude::*;

pub(crate) fn project_line(name: String) -> impl IntoView {
    a().href(format!("/projects/{name}")).child(name)
}

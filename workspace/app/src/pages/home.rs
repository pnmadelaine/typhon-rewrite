use crate::prelude::*;

#[server(prefix = "/api/leptos", endpoint = "get_projects")]
async fn get_projects() -> Result<Vec<String>, ServerFnError> {
    Ok(init()?.projects().await)
}

pub(crate) fn view() -> impl IntoView {
    (
        h2().child("Projects"),
        future_view(get_projects(), |projects| {
            ul().child(
                projects
                    .iter()
                    .cloned()
                    .map(|name| li().child(components::project_line(name)))
                    .collect_view(),
            )
        }),
    )
}

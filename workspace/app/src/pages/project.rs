use crate::prelude::*;

#[server(prefix = "/api/leptos", endpoint = "delete_project")]
async fn delete_project(name: String) -> Result<(), ServerFnError> {
    init()?.delete_project(name).await?;
    leptos_axum::redirect("/");
    Ok(())
}

fn instance(name: String) -> impl IntoView {
    (
        // Title
        h2().child(format!("Project {name}")),
        // Edit source
        components::source_edit(name.clone()),
        // Delete project
        components::button_fn("Delete".to_owned(), move || delete_project(name.clone())),
    )
}

pub(crate) fn view() -> impl IntoView {
    use_params_map().get().get("project_name").map(instance)
}

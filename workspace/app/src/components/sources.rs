use crate::prelude::*;

#[cfg(feature = "ssr")]
async fn set_source_inner(
    project_name: String,
    source: typhon_types::Source,
) -> Result<(), ServerFnError> {
    Ok(init()?
        .project(project_name)
        .await?
        .set_source(source)
        .await?)
}

pub(crate) fn source_edit(project_name: String) -> impl IntoView {
    #[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, FromStr)]
    enum Kind {
        #[default]
        GitHub,
    }
    let (kind, set_kind) = signal(Kind::GitHub);
    (
        select()
            .attr("selected", "github")
            .on_target(ev::change, move |ev| {
                set_kind.set(ev.target().value().parse().unwrap_or_default())
            })
            .child((option().value("github").child("GitHub"),)),
        move || match kind.get() {
            Kind::GitHub => {
                #[server(prefix = "/api/leptos", endpoint = "new_project_github")]
                async fn set_source(
                    project_name: String,
                    owner: String,
                    repo: String,
                    branch: String,
                    revision: String,
                ) -> Result<(), ServerFnError> {
                    let branch = (!branch.is_empty()).then(|| branch);
                    let revision = (!revision.is_empty()).then(|| revision);
                    set_source_inner(
                        project_name.clone(),
                        typhon_types::Source::GitHub {
                            owner,
                            repo,
                            branch,
                            revision,
                        },
                    )
                    .await?;
                    Ok(())
                }
                let action = ServerAction::<SetSource>::new();
                let project_name = project_name.clone();
                ActionForm(ActionFormProps {
                    action,
                    node_ref: None,
                    children: Box::new(|| {
                        (
                            input_t("hidden").value(project_name),
                            input_t("text").name("owner".to_owned()),
                            input_t("text").name("repo".to_owned()),
                            input_t("text").name("branch".to_owned()),
                            input_t("text").name("revision".to_owned()),
                            input_t("submit").value("Set source"),
                        )
                            .into_any()
                    }),
                })
            }
        },
    )
}

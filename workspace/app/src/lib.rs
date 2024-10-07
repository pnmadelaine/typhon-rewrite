mod components;
mod pages;
mod prelude;

use leptos::{children::ToChildren, tachys::html::doctype};
use leptos_router::{components::*, path, NestedRoute};

use crate::prelude::*;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(app);
}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    (
        doctype("html"),
        html().child((
            head().child(HydrationScripts(HydrationScriptsProps {
                options,
                islands: false,
                root: None,
            })),
            body().child(app()),
        )),
    )
}

pub fn app() -> impl IntoView {
    (Router(RouterProps {
        base: None,
        set_is_routing: None,
        children: TypedChildren::to_children(|| {
            (
                header().child(h1().child(a().href("/").child("Typhon"))),
                routes(),
            )
        }),
    }),)
}

fn routes() -> impl IntoView {
    use pages::*;
    fn r<Segments, View>(path: Segments, view: View) -> NestedRoute<Segments, (), (), View>
    where
        View: leptos_router::ChooseView,
    {
        NestedRoute::new(path, view)
    }
    let routes = (
        r(path!("/"), home::view),
        r(path!("/projects/:project_name"), project::view),
    );
    Routes(RoutesProps {
        fallback: || "Not found",
        transition: true, // TODO: what's this?
        children: RouteChildren::to_children(|| routes),
    })
}

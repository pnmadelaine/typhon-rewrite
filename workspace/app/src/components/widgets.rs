use crate::prelude::*;

pub(crate) fn button_fn<F, O>(text: String, f: F) -> impl IntoView
where
    F: FnOnce() -> O + Clone + 'static,
    O: std::future::Future<Output = Result<(), ServerFnError>> + 'static,
{
    button()
        .on_target(ev::click, move |_| {
            let f = f.clone();
            spawn_local(async {
                let _ = f().await;
            })
        })
        .child(text)
}

pub(crate) use derive_more::FromStr;
pub(crate) use leptos::{ev, html::*, prelude::*, task::spawn_local};
pub(crate) use leptos_router::hooks::use_params_map;
pub(crate) use serde::{de::DeserializeOwned, Deserialize, Serialize};
#[cfg(feature = "ssr")]
pub(crate) use typhon_core as core;

pub(crate) use crate::components;

#[cfg(feature = "ssr")]
pub(crate) fn init() -> Result<core::init::Addr, ServerFnError> {
    use_context().ok_or_else(|| ServerFnError::new("No context"))
}

pub(crate) fn future_view<C, T, Err, Fut, F>(future: Fut, child: F) -> impl IntoView
where
    T: Send + Sync + serde::Serialize + for<'de> serde::Deserialize<'de> + 'static,
    Err: Send + Sync + Serialize + DeserializeOwned + 'static,
    C: IntoView + 'static,
    Fut: std::future::Future<Output = Result<T, Err>> + Send + 'static,
    F: Fn(&T) -> C + Send + Sync + 'static,
{
    use leptos::either::Either;
    Await(AwaitProps {
        blocking: false,
        future,
        children: move |res| match res {
            Ok(x) => Either::Left(child(x)),
            Err(_err) => Either::Right("ERROR"),
        },
    })
}

pub fn input_t<V>(value: V) -> HtmlElement<Input, (leptos::attr::Attr<leptos::attr::Type, V>,), ()>
where
    V: leptos::attr::AttributeValue,
{
    input().r#type(value)
}

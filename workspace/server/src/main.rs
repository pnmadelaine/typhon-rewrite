mod api;

use leptos::config::LeptosOptions;
use leptos_axum::LeptosRoutes;
use typhon_core::*;

fn site_root() -> Option<std::path::PathBuf> {
    std::env::var("SITE_ROOT")
        .map(std::path::PathBuf::from)
        .ok()
        .or_else(|| {
            Some(
                std::env::current_exe()
                    .ok()?
                    .parent()?
                    .parent()?
                    .join("site"),
            )
        })
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Spawn Typhon.
    let database_url = std::env::var("DATABASE_URL").unwrap();
    run_pending_migrations(&database_url).await;
    repair_database(&database_url).await;
    let (init, handle) = typhon(Settings { database_url });

    // Build web server...
    let router = axum::Router::new();

    // ... add REST API.
    let router = router.nest("/api", api::api_routes(init.clone()));

    // ... add Leptos routes.
    let site_root = site_root().unwrap();
    let site_root = site_root.as_os_str().to_str().unwrap();
    let options = LeptosOptions::builder()
        .output_name("typhon")
        .site_root(site_root)
        .build();
    let ctx = {
        let init = init.clone();
        move || {
            leptos::context::provide_context(init.clone());
        }
    };
    let router = router
        .leptos_routes_with_context(
            &options,
            leptos_axum::generate_route_list(typhon_app::app),
            ctx.clone(),
            {
                let options = options.clone();
                move || typhon_app::shell(options.clone())
            },
        )
        .route(
            "/api/leptos/*fn_name",
            axum::routing::post({
                let ctx = ctx.clone();
                move |req| leptos_axum::handle_server_fns_with_context(ctx.clone(), req)
            }),
        )
        .fallback(leptos_axum::file_and_error_handler(typhon_app::shell))
        .with_state(options);
    drop(ctx);

    // Run web server.
    let listener = tokio::net::TcpListener::bind("localhost:3000")
        .await
        .unwrap();
    let server = axum::serve(listener, router).with_graceful_shutdown(shutdown_signal());

    // TODO: check that the connection to the database was successful.

    // Greetings.
    eprintln!("🐍⚡ Typhon is running.");

    // Wait for shutdown signal.
    tokio::select! {
        _ = handle => (),
        _ = server => (),
    };

    // Try to shutdown gracefully.
    eprintln!("");
    eprintln!("🐍⚡ Typhon is shutting down...");
    init.wait().await;
}

async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

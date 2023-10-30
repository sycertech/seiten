use std::net::SocketAddr;

use anyhow::Result;
use axum::Server;
use seiten::{
    config::{get_config, CONFIG},
    create_router,
};
use tracing_subscriber::{fmt, prelude::*, EnvFilter, Registry};

#[tokio::main]
async fn main() -> Result<()> {
    let _ = get_config();

    Registry::default()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug,tower_http=debug,axum::rejection=trace".into()),
        )
        .with(fmt::layer())
        .init();

    let app = create_router();

    let addr = SocketAddr::from(([0, 0, 0, 0], CONFIG.port));
    tracing::info!("Listening on http://{addr}");

    let _ = Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
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

    tracing::info!("signal received, starting graceful shutdown");
}

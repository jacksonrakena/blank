use std::collections::HashMap;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use axum::{Extension, Router};
use blank_parse::rules::Rule;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::add_extension::AddExtensionLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use log::info;

pub async fn execute_rule(
    Path(rule): Path<String>,
    Extension(targets): Extension<Arc<HashMap<String,Rule>>>,
) -> impl IntoResponse {

    if let Some(rule) = targets.get(&rule) {
        info!("executing {:?}", rule);
        return Redirect::to(&rule.url).into_response()
    }

    (StatusCode::NOT_FOUND, format!("no rule found for '{}'", rule)).into_response()
}
#[tokio::main]
async fn main() -> miette::Result<()> {
    let filter = Targets::new()
        .with_default(tracing_subscriber::filter::LevelFilter::INFO);

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();

    let targets = blank_parse::parse_targets()?;
    for (_, rule) in &targets {
        info!("Loaded {:?}", rule);
    }

    let app = Router::new()
        // Users
        .route("/{rule}", get(execute_rule))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .layer(AddExtensionLayer::new(Arc::new(targets)));

    let server = axum::serve(TcpListener::bind("0.0.0.0:3000").await.unwrap(), app);

    if let Err(err) = server.await {
        miette::bail!("server error: {}", err);
    }

    Ok(())
}

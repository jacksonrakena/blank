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

pub async fn execute_rule(
    Path(rule): Path<String>,
    Extension(targets): Extension<Arc<Vec<Rule>>>,
) -> impl IntoResponse {
    for target in targets.iter() {
        if target.name == rule {
            println!("Executing rule '{}' with URL: {}", rule, target.url);
            return Redirect::to(&target.url).into_response()
        }
    }

    (StatusCode::NOT_FOUND, format!("no rule found for '{}'", rule)).into_response()
}
#[tokio::main]
async fn main() -> miette::Result<()> {
    let targets = blank_parse::parse_targets()?;
    for (name, rule) in &targets {
        println!("{:?}", rule);
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

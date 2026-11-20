//! Supplier Agent — presentation adapter over application supplier use case.

use anyhow::Result;
use clap::Parser;
use coop_a2a::card::AgentCard;
use coop_a2a::models::*;
use coop_a2a::server::{completed_task, router, AppState};
use coop_application::supplier_brief;
use coop_identity::ApiKeyDirectory;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value = "127.0.0.1:7102")]
    addr: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let args = Args::parse();
    let card = AgentCard::supplier(format!("http://{}", args.addr));
    let auth = Arc::new(ApiKeyDirectory::demo());

    let handler: coop_a2a::server::SkillHandler = Arc::new(|_principal, skill, message| {
        Box::pin(async move {
            if skill != "supplier.read" {
                return Err((
                    axum::http::StatusCode::BAD_REQUEST,
                    format!("unknown skill `{skill}`"),
                ));
            }

            let params = message.data_part().unwrap_or(SupplierLookupParams {
                supplier_id: "unknown".into(),
                category: None,
            });

            let brief = supplier_brief(&params.supplier_id, params.category.as_deref());
            Ok(completed_task(
                "supplier.read",
                Artifact {
                    name: "supplier_brief".into(),
                    mime_type: "application/json".into(),
                    data: brief,
                },
                Message {
                    role: "agent".into(),
                    parts: vec![Part::Text {
                        text: format!("Supplier brief for {}", params.supplier_id),
                    }],
                },
            ))
        })
    });

    let app = router(AppState {
        card,
        auth,
        handler,
    });
    tracing::info!("Supplier Agent listening on http://{}", args.addr);
    axum::serve(tokio::net::TcpListener::bind(args.addr).await?, app).await?;
    Ok(())
}

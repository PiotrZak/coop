//! Negotiation Agent — presentation adapter over application negotiation use case.

use anyhow::Result;
use clap::Parser;
use coop_a2a::card::AgentCard;
use coop_a2a::client::A2aClient;
use coop_a2a::models::*;
use coop_a2a::server::{completed_task, router, AppState};
use coop_application::build_negotiation_pack;
use coop_identity::ApiKeyDirectory;
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value = "negotiation-b")]
    name: String,
    #[arg(long, default_value = "127.0.0.1:7101")]
    addr: SocketAddr,
    #[arg(long, default_value = "http://127.0.0.1:7102")]
    supplier_url: String,
    #[arg(long, default_value = "nego-b-demo-key")]
    supplier_bearer: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let args = Args::parse();
    let card = AgentCard::negotiation(format!("http://{}", args.addr), &args.name);
    let auth = Arc::new(ApiKeyDirectory::demo());
    let supplier = A2aClient::new(args.supplier_url.clone(), args.supplier_bearer.clone());
    let agent_name = args.name.clone();

    let handler: coop_a2a::server::SkillHandler = Arc::new(move |principal, skill, message| {
        let supplier = supplier.clone();
        let agent_name = agent_name.clone();
        Box::pin(async move {
            if skill != "negotiate" {
                return Err((
                    axum::http::StatusCode::BAD_REQUEST,
                    format!("unknown skill `{skill}`"),
                ));
            }

            let params = message.data_part().unwrap_or(NegotiateParams {
                counterparty: "unknown".into(),
                category: "general".into(),
                ask: "(empty)".into(),
                supplier_id: None,
            });

            let supplier_artifact = if let Some(ref sid) = params.supplier_id {
                match supplier
                    .supplier_lookup(SupplierLookupParams {
                        supplier_id: sid.clone(),
                        category: Some(params.category.clone()),
                    })
                    .await
                {
                    Ok(task) => task.artifact.map(|a| a.data),
                    Err(e) => {
                        tracing::warn!("supplier consult failed: {e}");
                        Some(json!({ "error": e.to_string() }))
                    }
                }
            } else {
                None
            };

            let (artifact, reply) =
                build_negotiation_pack(&agent_name, &principal, &params, supplier_artifact);
            Ok(completed_task("negotiate", artifact, reply))
        })
    });

    let app = router(AppState {
        card,
        auth,
        handler,
    });
    tracing::info!("{} listening on http://{}", args.name, args.addr);
    axum::serve(tokio::net::TcpListener::bind(args.addr).await?, app).await?;
    Ok(())
}

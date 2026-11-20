//! Negotiation Agent A2A edge.
//!
//! Accepts peer Negotiation `negotiate` calls. When `supplier_id` is present,
//! consults the Supplier Agent over A2A before returning a pack artifact.

use anyhow::Result;
use clap::Parser;
use coop_a2a::card::AgentCard;
use coop_a2a::client::A2aClient;
use coop_a2a::models::*;
use coop_a2a::server::{completed_task, router, AppState};
use coop_identity::ApiKeyDirectory;
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, env = "COOP_NEGO_NAME", default_value = "negotiation-a")]
    name: String,

    #[arg(long, env = "COOP_NEGO_ADDR", default_value = "127.0.0.1:7100")]
    addr: SocketAddr,

    /// Supplier Agent base URL for consults.
    #[arg(
        long,
        env = "COOP_SUPPLIER_URL",
        default_value = "http://127.0.0.1:7102"
    )]
    supplier_url: String,

    /// Bearer key used when this Negotiation Agent calls Supplier.
    #[arg(long, env = "COOP_SUPPLIER_BEARER", default_value = "nego-a-demo-key")]
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

            let params = extract_negotiate(&message).unwrap_or(NegotiateParams {
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

            let pack = json!({
                "from_agent": agent_name,
                "caller": principal.agent_id,
                "caller_tenant": principal.tenant_id,
                "counterparty": params.counterparty,
                "category": params.category,
                "ask": params.ask,
                "supplier_consult": supplier_artifact,
                "options": [
                    { "label": "Hold price + volume commit", "risk": "low" },
                    { "label": "Ask 8–12% reduction", "risk": "medium" },
                    { "label": "Dual-source threat", "risk": "high" },
                ],
                "talking_points": [
                    "Use supplier brief leverage only if present",
                    "HITL required before any external send",
                ],
                "hitl": true,
            });

            Ok(completed_task(
                "negotiate",
                Artifact {
                    name: "negotiation_pack".into(),
                    mime_type: "application/json".into(),
                    data: pack,
                },
                Message {
                    role: "agent".into(),
                    parts: vec![Part::Text {
                        text: format!(
                            "Negotiation pack from {} (supplier consult: {})",
                            agent_name,
                            params.supplier_id.as_deref().unwrap_or("none")
                        ),
                    }],
                },
            ))
        })
    });

    let state = AppState {
        card,
        auth,
        handler,
    };
    let app = router(state);
    tracing::info!(
        "{} Negotiation Agent listening on http://{}",
        args.name,
        args.addr
    );
    let listener = tokio::net::TcpListener::bind(args.addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn extract_negotiate(message: &Message) -> Option<NegotiateParams> {
    for part in &message.parts {
        if let Part::Data { data } = part {
            return serde_json::from_value(data.clone()).ok();
        }
    }
    None
}

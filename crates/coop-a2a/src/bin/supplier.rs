//! Supplier Agent A2A edge — returns facts for a particular supplier_id.

use anyhow::Result;
use clap::Parser;
use coop_a2a::card::AgentCard;
use coop_a2a::models::*;
use coop_a2a::server::{completed_task, router, AppState};
use coop_identity::ApiKeyDirectory;
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, env = "COOP_SUPPLIER_ADDR", default_value = "127.0.0.1:7102")]
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

            let params = extract_supplier_params(&message).unwrap_or(SupplierLookupParams {
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

    let state = AppState {
        card,
        auth,
        handler,
    };
    let app = router(state);
    tracing::info!("Supplier Agent listening on http://{}", args.addr);
    let listener = tokio::net::TcpListener::bind(args.addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn extract_supplier_params(message: &Message) -> Option<SupplierLookupParams> {
    for part in &message.parts {
        if let Part::Data { data } = part {
            return serde_json::from_value(data.clone()).ok();
        }
    }
    None
}

fn supplier_brief(supplier_id: &str, category: Option<&str>) -> serde_json::Value {
    match supplier_id {
        "acme-cloud" => json!({
            "supplier_id": "acme-cloud",
            "name": "Acme Cloud",
            "category": category.unwrap_or("SaaS"),
            "annual_spend": 1_200_000,
            "leverage": "high",
            "renewal_days": 45,
            "notes": ["multi-year discount unused", "competitor quote available"],
        }),
        "nordic-steel" => json!({
            "supplier_id": "nordic-steel",
            "name": "Nordic Steel",
            "category": category.unwrap_or("Raw materials"),
            "annual_spend": 840_000,
            "leverage": "medium",
            "renewal_days": 120,
            "notes": ["volume commitment missed last year"],
        }),
        other => json!({
            "supplier_id": other,
            "name": other,
            "category": category,
            "annual_spend": null,
            "leverage": "unknown",
            "notes": ["no local demo record — wire to Lindle Supplier skill"],
        }),
    }
}

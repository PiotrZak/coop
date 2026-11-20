//! Demo client: call coop-gateway negotiate skill (backed by Lindle).

use anyhow::Result;
use clap::Parser;
use coop_a2a::client::A2aClient;
use coop_a2a::models::NegotiateParams;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value = "http://127.0.0.1:7090")]
    peer_url: String,

    #[arg(long, default_value = "nego-a-demo-key")]
    bearer: String,

    #[arg(long, default_value = "acme-cloud")]
    supplier_id: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let args = Args::parse();
    let client = A2aClient::new(&args.peer_url, &args.bearer);

    let card = client.get_agent_card().await?;
    println!("Peer Agent Card: {} @ {}", card.name, card.url);

    let task = client
        .negotiate(NegotiateParams {
            counterparty: "Acme Cloud sales".into(),
            category: "SaaS".into(),
            ask: "Prepare options for renewal down 10%".into(),
            supplier_id: Some(args.supplier_id),
        })
        .await?;

    println!("Task {} status={:?}", task.id, task.status);
    if let Some(artifact) = task.artifact {
        println!("{}", serde_json::to_string_pretty(&artifact.data)?);
    }
    Ok(())
}

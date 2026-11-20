use coop_domain::{Artifact, Message, NegotiateParams, Part, Principal};
use serde_json::{json, Value};

/// Build HITL negotiation pack artifact + agent reply.
pub fn build_negotiation_pack(
    agent_name: &str,
    principal: &Principal,
    params: &NegotiateParams,
    supplier_consult: Option<Value>,
) -> (Artifact, Message) {
    let artifact = Artifact {
        name: "negotiation_pack".into(),
        mime_type: "application/json".into(),
        data: json!({
            "from_agent": agent_name,
            "caller": principal.agent_id,
            "caller_tenant": principal.tenant_id,
            "counterparty": params.counterparty,
            "category": params.category,
            "ask": params.ask,
            "supplier_consult": supplier_consult,
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
        }),
    };
    let message = Message {
        role: "agent".into(),
        parts: vec![Part::Text {
            text: format!(
                "Negotiation pack from {agent_name} (supplier consult: {})",
                params.supplier_id.as_deref().unwrap_or("none")
            ),
        }],
    };
    (artifact, message)
}

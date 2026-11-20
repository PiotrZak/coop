//! Supplier brief use case.

use serde_json::{json, Value};

pub fn supplier_brief(supplier_id: &str, category: Option<&str>) -> Value {
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

//! Routing + gateway card merge.

use coop_a2a::card::AgentCard;
use coop_a2a::proxy::{build_gateway_card, merge_skills, SkillRouter};
use coop_domain::Scope;

#[test]
fn skill_router_lookup() {
    let r = SkillRouter::demo();
    assert!(r.get("negotiate").is_some());
    assert!(r.get("unknown").is_none());
}

#[test]
fn merge_skills_dedupes() {
    let skills = merge_skills([
        AgentCard::negotiation("http://a", "nego"),
        AgentCard::supplier("http://b"),
        AgentCard::negotiation("http://c", "dup"),
    ]);
    assert_eq!(skills.len(), 2);
    assert_eq!(skills[0].scopes, vec![Scope::Negotiate]);
}

#[test]
fn gateway_card_name() {
    let card = build_gateway_card(
        "http://127.0.0.1:7090",
        vec![
            AgentCard::negotiation("http://n", "n"),
            AgentCard::supplier("http://s"),
        ],
    );
    assert_eq!(card.name, "coop-gateway");
    assert_eq!(card.skills.len(), 2);
}

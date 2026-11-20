use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRoute {
    pub skill: String,
    pub upstream_url: String,
    pub upstream_bearer: String,
}

#[derive(Debug, Clone, Default)]
pub struct SkillRouter {
    routes: HashMap<String, SkillRoute>,
}

impl SkillRouter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, route: SkillRoute) {
        self.routes.insert(route.skill.clone(), route);
    }

    pub fn demo() -> Self {
        let mut r = Self::new();
        for (skill, url) in [
            ("negotiate", "http://127.0.0.1:7101"),
            ("supplier.read", "http://127.0.0.1:7102"),
        ] {
            r.insert(SkillRoute {
                skill: skill.into(),
                upstream_url: url.into(),
                upstream_bearer: "gateway-service-key".into(),
            });
        }
        r
    }

    pub fn get(&self, skill: &str) -> Option<&SkillRoute> {
        self.routes.get(skill)
    }

    pub fn skills(&self) -> impl Iterator<Item = &str> {
        self.routes.keys().map(String::as_str)
    }

    pub fn routes(&self) -> impl Iterator<Item = &SkillRoute> {
        self.routes.values()
    }
}

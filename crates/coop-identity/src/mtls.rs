//! mTLS via TLS-terminator proxy headers.

use anyhow::{anyhow, Result};
use coop_domain::{Principal, Role};
use http::HeaderMap;
use std::collections::HashMap;
use std::env;

#[derive(Debug, Clone, Default)]
pub struct MtlsDirectory {
    by_subject: HashMap<String, Principal>,
}

impl MtlsDirectory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, subject_or_cn: impl Into<String>, principal: Principal) {
        self.by_subject.insert(subject_or_cn.into(), principal);
    }

    pub fn demo() -> Self {
        let mut d = Self::new();
        let p = Principal {
            agent_id: "negotiation-a".into(),
            tenant_id: "buyer-tenant".into(),
            roles: vec![Role::Operator],
        };
        d.register("nego-a.agents.local", p.clone());
        d.register("CN=nego-a.agents.local", p);
        d
    }

    pub fn from_env() -> Option<Self> {
        let flag = env::var("COOP_MTLS_PROXY_HEADERS").ok()?;
        matches!(flag.to_ascii_lowercase().as_str(), "1" | "true" | "yes").then(Self::demo)
    }

    /// `Ok(None)` if headers do not indicate a verified client cert.
    pub fn try_authenticate(&self, headers: &HeaderMap) -> Result<Option<Principal>> {
        if !verified(headers) {
            return Ok(None);
        }
        let subject = subject(headers)
            .ok_or_else(|| anyhow!("mTLS verified but missing client subject header"))?;
        let cn = extract_cn(subject).unwrap_or(subject);
        self.by_subject
            .get(cn)
            .or_else(|| self.by_subject.get(subject))
            .cloned()
            .map(Some)
            .ok_or_else(|| anyhow!("mTLS subject `{subject}` not registered"))
    }
}

fn verified(headers: &HeaderMap) -> bool {
    eq(headers, "x-ssl-client-verify", "SUCCESS")
        || eq(headers, "x-ssl-client-verify", "success")
        || eq(headers, "x-forwarded-tls-client-cert-verify", "SUCCESS")
        || eq(headers, "x-client-cert-verified", "true")
        || eq(headers, "x-client-cert-verified", "1")
}

fn subject(headers: &HeaderMap) -> Option<&str> {
    for name in [
        "x-ssl-client-s-dn",
        "x-ssl-client-dn",
        "x-forwarded-client-cert-dn",
        "x-client-cert-dn",
        "x-client-cert-subject",
    ] {
        if let Some(v) = headers.get(name).and_then(|v| v.to_str().ok()).map(str::trim) {
            if !v.is_empty() {
                return Some(v);
            }
        }
    }
    None
}

fn eq(headers: &HeaderMap, name: &str, expected: &str) -> bool {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.trim().eq_ignore_ascii_case(expected))
}

fn extract_cn(dn: &str) -> Option<&str> {
    dn.split(',').find_map(|part| {
        let part = part.trim();
        part.strip_prefix("CN=")
            .or_else(|| part.strip_prefix("cn="))
            .map(str::trim)
    })
}

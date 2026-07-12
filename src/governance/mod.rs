//! Governance — Microsoft Agent Governance Toolkit (AGT) integration.
//!
//! Phase 0 of the AGT integration plan. This module currently exists only to
//! prove the `agentmesh` v3.x crate links against caro's workspace (MSRV 1.85,
//! edition 2021, AGPL-3.0). It does **not** yet wrap [`crate::safety`] or
//! emit audit events from [`crate::agent`] — those land in Phase 1+.
//!
//! See `.claude/plans/intgrate-https-github-com-microsoft-agen-witty-scroll.md`
//! for the full architectural plan.
//!
//! # Cargo feature
//!
//! Gated behind the `governance` Cargo feature, which pulls in `agentmesh`
//! as an optional dependency. The whole module is `#[cfg(feature =
//! "governance")]` in `lib.rs`, so a default build with `--no-default-features`
//! never sees the AGT vocabulary.

/// Phase 0 smoke check. Constructs an [`agentmesh::AgentMeshClient`] to verify
/// the crate is linkable in caro's workspace. Returns `Ok(())` on success.
///
/// This function is not part of any user-facing flow. It is referenced by the
/// build verification step in the Phase 0 PR and exists solely so cargo
/// actually compiles the `agentmesh` crate (an unused optional dep would not
/// be compiled, which defeats the purpose of the spike).
///
/// # Errors
///
/// Returns the underlying `agentmesh` error if client construction fails.
pub fn build_spike() -> Result<(), Box<dyn std::error::Error>> {
    let _client = agentmesh::AgentMeshClient::new("caro-governance-spike")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::build_spike;

    #[test]
    fn build_spike_constructs_client() {
        build_spike().expect("agentmesh client should construct in test env");
    }
}

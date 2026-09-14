//! Deriving a tenant's database id from its Zitadel organization id. ADR-0082.
//!
//! ADR-0099 makes docket multi-tenant: the served backend reads the tenant from the operator token
//! rather than a fixed `DOCKET_TENANT_ID`, and the token carries the org as the `resourceowner`
//! snowflake. The database keys tenants by a uuid, and ADR-0082 D1 makes that uuid a pure function
//! of the org id — `uuidv5(namespace, orgId)` — so a request resolves its tenant with no database
//! lookup (login must work before the store holds data; the derivation cannot depend on it).
//!
//! The SAME value is computed by `web-admin`'s `deriveTenantId` (TypeScript) and by
//! `platform-tenancy`'s provisioning (`uuid.uuid5` in Python) when it seeds the `tenants` row —
//! same namespace, same org id, same uuid on every side. The namespace is FIXED FOREVER; changing
//! it re-keys every tenant.

use uuid::Uuid;

/// The ADR-0082 namespace. Never change it.
const NAMESPACE: Uuid = Uuid::from_bytes([
    0xca, 0xed, 0x83, 0x97, 0x80, 0xa3, 0x48, 0x60, 0x82, 0x47, 0xdf, 0x68, 0xb7, 0xde, 0x8f, 0xd3,
]);

/// The tenant's database id derived from its Zitadel organization id (the `resourceowner` claim).
/// This is the value that becomes `paridyum.tenant_id` for the request. ADR-0082.
///
/// # Errors
/// Fails when the org id is empty — a token with no `resourceowner` names no tenant, and defaulting
/// one would be the multi-tenant equivalent of the `DOCKET_TENANT_ID` guess ADR-0099 removes.
pub fn derive_tenant_id(zitadel_org_id: &str) -> anyhow::Result<Uuid> {
    anyhow::ensure!(
        !zitadel_org_id.trim().is_empty(),
        "the token carries no resourceowner (org id); a multi-tenant request must name its tenant"
    );
    Ok(Uuid::new_v5(&NAMESPACE, zitadel_org_id.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paridyum_org_derives_the_known_tenant_id() {
        // Proven cross-language: Python `uuid.uuid5(NAMESPACE, "389623583590057140")` and
        // web-admin's `deriveTenantId` both produce this, and it is the tenant id the live cluster
        // was seeded with. A mismatch here would silently isolate Paridyum from its own rows.
        let derived = derive_tenant_id("389623583590057140").expect("non-empty org id");
        assert_eq!(
            derived,
            Uuid::parse_str("68445ee9-175a-58a9-834e-a2f0c9c9b124").unwrap()
        );
    }

    #[test]
    fn the_namespace_constant_is_the_adr_0082_uuid() {
        assert_eq!(
            NAMESPACE,
            Uuid::parse_str("caed8397-80a3-4860-8247-df68b7de8fd3").unwrap()
        );
    }

    #[test]
    fn an_empty_org_id_is_refused() {
        assert!(derive_tenant_id("").is_err());
        assert!(derive_tenant_id("   ").is_err());
    }

    #[test]
    fn derivation_is_deterministic_and_distinct_per_org() {
        let a = derive_tenant_id("389623583590057140").unwrap();
        let a_again = derive_tenant_id("389623583590057140").unwrap();
        let b = derive_tenant_id("111111111111111111").unwrap();
        assert_eq!(a, a_again);
        assert_ne!(a, b);
    }
}

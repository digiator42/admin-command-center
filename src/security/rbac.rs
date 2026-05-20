
use gritshield::routing::trie::RequestContext;

pub trait RbacExtensions {
    fn get_user_role(&self) -> Option<String>;
    fn has_role(&self, target_role: &str) -> bool;
    fn require_role(&self, target_role: &str) -> bool;
}

impl RbacExtensions for RequestContext {
    /// Extracts the cached role string natively out of GritShield's thread-safe state store
    fn get_user_role(&self) -> Option<String> {
        self.session.as_ref().and_then(|session_arc| {
            let session = session_arc.lock().unwrap();
            session.data.get("role").cloned()
        })
    }

    /// Non-blocking check to evaluate permissions
    fn has_role(&self, target_role: &str) -> bool {
        match self.get_user_role() {
            Some(role) => {
                // Hierarchical authorization structure logic
                match (role.as_str(), target_role) {
                    ("SuperAdmin", _) => true, // bypass
                    ("Operator", "SuperAdmin") => false,
                    ("Operator", _) => true,   // Operators access standard and low tiers
                    ("Auditor", "Auditor") => true,
                    _ => false,
                }
            }
            None => false,
        }
    }

    /// Strict guard line item helper to immediately throw clean execution flags
    fn require_role(&self, target_role: &str) -> bool {
        self.has_role(target_role)
    }
}
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    ReadTarget,
    BuildTarget,
    DeployTarget,
    AdminTarget,
    SignArtifact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub permissions: HashSet<Permission>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityClaims {
    pub subject: String,
    pub email: String,
    pub roles: Vec<String>,
    pub issuer: String,
    pub expires_at: u64,
}

pub struct AccessController {
    roles: HashMap<String, Role>,
}

impl AccessController {
    pub fn new() -> Self {
        let mut controller = Self {
            roles: HashMap::new(),
        };

        let mut developer_perms = HashSet::new();
        developer_perms.insert(Permission::ReadTarget);
        developer_perms.insert(Permission::BuildTarget);
        controller.register_role("developer", developer_perms);

        let mut release_perms = HashSet::new();
        release_perms.insert(Permission::ReadTarget);
        release_perms.insert(Permission::BuildTarget);
        release_perms.insert(Permission::DeployTarget);
        release_perms.insert(Permission::SignArtifact);
        controller.register_role("release-manager", release_perms);

        let mut admin_perms = HashSet::new();
        admin_perms.insert(Permission::ReadTarget);
        admin_perms.insert(Permission::BuildTarget);
        admin_perms.insert(Permission::DeployTarget);
        admin_perms.insert(Permission::AdminTarget);
        admin_perms.insert(Permission::SignArtifact);
        controller.register_role("admin", admin_perms);

        controller
    }

    pub fn register_role(&mut self, name: &str, permissions: HashSet<Permission>) {
        self.roles.insert(
            name.to_string(),
            Role {
                name: name.to_string(),
                permissions,
            },
        );
    }

    pub fn check_permission(&self, claims: &IdentityClaims, required: Permission) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        if claims.expires_at > 0 && claims.expires_at < now {
            return false;
        }

        for role_name in &claims.roles {
            if let Some(role) = self.roles.get(role_name) {
                if role.permissions.contains(&required) {
                    return true;
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rbac_access_control() {
        let ac = AccessController::new();

        let claims = IdentityClaims {
            subject: "user_123".to_string(),
            email: "dev@company.com".to_string(),
            roles: vec!["developer".to_string()],
            issuer: "https://auth.company.com".to_string(),
            expires_at: 9999999999,
        };

        assert!(ac.check_permission(&claims, Permission::BuildTarget));
        assert!(!ac.check_permission(&claims, Permission::AdminTarget));
    }
}

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
    ReadSecrets,
    WriteSecrets,
    ManageWorkers,
    ViewAnalytics,
    ManagePlugins,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub permissions: HashSet<Permission>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityClaims {
    pub subject: String,
    pub email: String,
    pub roles: Vec<String>,
    pub issuer: String,
    pub audience: String,
    pub expires_at: u64,
    pub issued_at: u64,
    pub groups: Vec<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_url: String,
    pub scopes: Vec<String>,
    pub enable_pkce: bool,
}

impl Default for OidcConfig {
    fn default() -> Self {
        Self {
            issuer_url: "https://auth.fish.build".to_string(),
            client_id: "fish-cli".to_string(),
            client_secret: None,
            redirect_url: "http://localhost:8080/callback".to_string(),
            scopes: vec!["openid".to_string(), "email".to_string(), "profile".to_string(), "groups".to_string()],
            enable_pkce: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub timestamp: u64,
    pub subject: String,
    pub email: String,
    pub action: String,
    pub target: String,
    pub permission: Permission,
    pub granted: bool,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub trace_id: Option<String>,
}

pub struct AccessController {
    roles: HashMap<String, Role>,
    audit_log: Vec<AuditLogEntry>,
    oidc_config: Option<OidcConfig>,
}

impl AccessController {
    pub fn new() -> Self {
        let mut controller = Self {
            roles: HashMap::new(),
            audit_log: Vec::new(),
            oidc_config: None,
        };

        // Developer role - basic build access
        let mut developer_perms = HashSet::new();
        developer_perms.insert(Permission::ReadTarget);
        developer_perms.insert(Permission::BuildTarget);
        developer_perms.insert(Permission::ViewAnalytics);
        controller.register_role_with_description(
            "developer",
            developer_perms,
            "Standard developer with build and read access",
        );

        // CI role - automated builds
        let mut ci_perms = HashSet::new();
        ci_perms.insert(Permission::ReadTarget);
        ci_perms.insert(Permission::BuildTarget);
        ci_perms.insert(Permission::ReadSecrets);
        controller.register_role_with_description(
            "ci",
            ci_perms,
            "CI/CD system with build and secret read access",
        );

        // Release manager - deploy and sign
        let mut release_perms = HashSet::new();
        release_perms.insert(Permission::ReadTarget);
        release_perms.insert(Permission::BuildTarget);
        release_perms.insert(Permission::DeployTarget);
        release_perms.insert(Permission::SignArtifact);
        release_perms.insert(Permission::ReadSecrets);
        release_perms.insert(Permission::ViewAnalytics);
        controller.register_role_with_description(
            "release-manager",
            release_perms,
            "Release manager with deploy and signing capabilities",
        );

        // Security auditor - read-only with audit access
        let mut auditor_perms = HashSet::new();
        auditor_perms.insert(Permission::ReadTarget);
        auditor_perms.insert(Permission::ViewAnalytics);
        controller.register_role_with_description(
            "auditor",
            auditor_perms,
            "Security auditor with read and analytics access",
        );

        // Admin - full access
        let mut admin_perms = HashSet::new();
        admin_perms.insert(Permission::ReadTarget);
        admin_perms.insert(Permission::BuildTarget);
        admin_perms.insert(Permission::DeployTarget);
        admin_perms.insert(Permission::AdminTarget);
        admin_perms.insert(Permission::SignArtifact);
        admin_perms.insert(Permission::ReadSecrets);
        admin_perms.insert(Permission::WriteSecrets);
        admin_perms.insert(Permission::ManageWorkers);
        admin_perms.insert(Permission::ViewAnalytics);
        admin_perms.insert(Permission::ManagePlugins);
        controller.register_role_with_description(
            "admin",
            admin_perms,
            "Administrator with full access to all resources",
        );

        controller
    }

    pub fn with_oidc_config(mut self, config: OidcConfig) -> Self {
        self.oidc_config = Some(config);
        self
    }

    pub fn register_role(&mut self, name: &str, permissions: HashSet<Permission>) {
        self.register_role_with_description(name, permissions, "");
    }

    pub fn register_role_with_description(
        &mut self,
        name: &str,
        permissions: HashSet<Permission>,
        description: &str,
    ) {
        self.roles.insert(
            name.to_string(),
            Role {
                name: name.to_string(),
                permissions,
                description: description.to_string(),
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

        // Also check groups mapping to roles
        for group in &claims.groups {
            if let Some(role) = self.roles.get(group) {
                if role.permissions.contains(&required) {
                    return true;
                }
            }
        }

        false
    }

    pub fn check_permission_with_audit(
        &mut self,
        claims: &IdentityClaims,
        required: Permission,
        target: &str,
        ip_address: Option<String>,
    ) -> bool {
        let granted = self.check_permission(claims, required.clone());
        
        let entry = AuditLogEntry {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            subject: claims.subject.clone(),
            email: claims.email.clone(),
            action: format!("{:?}", required),
            target: target.to_string(),
            permission: required,
            granted,
            ip_address,
            user_agent: None,
            trace_id: None,
        };

        self.audit_log.push(entry);
        granted
    }

    pub fn get_audit_log(&self) -> &[AuditLogEntry] {
        &self.audit_log
    }

    pub fn get_audit_log_for_user(&self, subject: &str) -> Vec<&AuditLogEntry> {
        self.audit_log
            .iter()
            .filter(|entry| entry.subject == subject)
            .collect()
    }

    pub fn get_failed_access_attempts(&self) -> Vec<&AuditLogEntry> {
        self.audit_log
            .iter()
            .filter(|entry| !entry.granted)
            .collect()
    }

    pub fn list_roles(&self) -> Vec<&Role> {
        self.roles.values().collect()
    }

    pub fn get_role(&self, name: &str) -> Option<&Role> {
        self.roles.get(name)
    }

    /// Simulate OIDC token validation
    pub fn validate_oidc_token(&self, token: &str) -> Result<IdentityClaims, String> {
        if token.is_empty() {
            return Err("empty token".to_string());
        }

        // In production, would validate JWT signature against OIDC provider's JWKS
        // For simulation, parse as JSON or check format
        if token.starts_with("eyJ") {
            // Looks like JWT, try to parse claims
            // For simulation, return mock claims
            Ok(IdentityClaims {
                subject: "user_123".to_string(),
                email: "user@example.com".to_string(),
                roles: vec!["developer".to_string()],
                issuer: self
                    .oidc_config
                    .as_ref()
                    .map(|c| c.issuer_url.clone())
                    .unwrap_or_else(|| "https://auth.fish.build".to_string()),
                audience: "fish-cli".to_string(),
                expires_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
                    + 3600,
                issued_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                groups: vec!["developers".to_string()],
                name: Some("Test User".to_string()),
            })
        } else {
            Err("invalid token format".to_string())
        }
    }

    /// Generate OIDC authorization URL
    pub fn get_authorization_url(&self, state: &str) -> Result<String, String> {
        let config = self
            .oidc_config
            .as_ref()
            .ok_or_else(|| "OIDC not configured".to_string())?;

        let scope = config.scopes.join(" ");
        Ok(format!(
            "{}/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}",
            config.issuer_url,
            config.client_id,
            config.redirect_url,
            scope,
            state
        ))
    }
}

impl Default for AccessController {
    fn default() -> Self {
        Self::new()
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
            audience: "fish-cli".to_string(),
            expires_at: 9999999999,
            issued_at: 0,
            groups: vec![],
            name: None,
        };

        assert!(ac.check_permission(&claims, Permission::BuildTarget));
        assert!(!ac.check_permission(&claims, Permission::AdminTarget));
        assert!(ac.check_permission(&claims, Permission::ViewAnalytics));
    }

    #[test]
    fn test_rbac_with_audit_logging() {
        let mut ac = AccessController::new();

        let claims = IdentityClaims {
            subject: "user_456".to_string(),
            email: "admin@company.com".to_string(),
            roles: vec!["admin".to_string()],
            issuer: "https://auth.company.com".to_string(),
            audience: "fish-cli".to_string(),
            expires_at: 9999999999,
            issued_at: 0,
            groups: vec![],
            name: Some("Admin User".to_string()),
        };

        assert!(ac.check_permission_with_audit(
            &claims,
            Permission::AdminTarget,
            "sensitive-build-target",
            Some("192.168.1.100".to_string())
        ));

        assert_eq!(ac.get_audit_log().len(), 1);
        assert_eq!(ac.get_audit_log()[0].subject, "user_456");
        assert!(ac.get_audit_log()[0].granted);

        // Failed attempt
        let dev_claims = IdentityClaims {
            subject: "user_789".to_string(),
            email: "dev@company.com".to_string(),
            roles: vec!["developer".to_string()],
            issuer: "https://auth.company.com".to_string(),
            audience: "fish-cli".to_string(),
            expires_at: 9999999999,
            issued_at: 0,
            groups: vec![],
            name: None,
        };

        assert!(!ac.check_permission_with_audit(
            &dev_claims,
            Permission::AdminTarget,
            "sensitive-target",
            None
        ));

        assert_eq!(ac.get_failed_access_attempts().len(), 1);
    }

    #[test]
    fn test_oidc_flow() {
        let oidc_config = OidcConfig {
            issuer_url: "https://auth.example.com".to_string(),
            client_id: "fish-test".to_string(),
            client_secret: None,
            redirect_url: "http://localhost:8080/callback".to_string(),
            scopes: vec!["openid".to_string(), "email".to_string()],
            enable_pkce: true,
        };

        let ac = AccessController::new().with_oidc_config(oidc_config);

        let auth_url = ac.get_authorization_url("random_state_123").unwrap();
        assert!(auth_url.contains("auth.example.com"));
        assert!(auth_url.contains("random_state_123"));

        // Simulate token validation
        let fake_jwt = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.signature";
        let claims = ac.validate_oidc_token(fake_jwt).unwrap();
        assert_eq!(claims.subject, "user_123");
    }

    #[test]
    fn test_group_based_access() {
        let ac = AccessController::new();

        let claims = IdentityClaims {
            subject: "user_group".to_string(),
            email: "groupuser@company.com".to_string(),
            roles: vec![], // No direct roles
            issuer: "https://auth.company.com".to_string(),
            audience: "fish-cli".to_string(),
            expires_at: 9999999999,
            issued_at: 0,
            groups: vec!["admin".to_string()], // But group is admin
            name: None,
        };

        // Should get access via group mapping
        assert!(ac.check_permission(&claims, Permission::AdminTarget));
    }
}

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rmcp::{RoleServer, model::CallToolResult, service::RequestContext};
use serde::Deserialize;
use std::collections::HashMap;

use crate::tools::error_result;

/// Represents the member permissions passed from the Lago API
/// The structure matches the Ruby permissions_hash format
#[derive(Debug, Clone, Deserialize, Default)]
pub struct MemberPermissions {
    #[serde(flatten)]
    permissions: HashMap<String, serde_json::Value>,
}

impl MemberPermissions {
    /// Check if a specific permission is granted
    /// Supports nested permissions like "customers:view" or "credit_notes:create"
    pub fn has_permission(&self, permission: &str) -> bool {
        let parts: Vec<&str> = permission.split(':').collect();
        if parts.is_empty() {
            return false;
        }

        let mut current_value: Option<&serde_json::Value> = self.permissions.get(parts[0]);

        for part in parts.iter().skip(1) {
            match current_value {
                Some(serde_json::Value::Object(obj)) => {
                    current_value = obj.get(*part);
                }
                Some(serde_json::Value::Bool(b)) => {
                    return *b;
                }
                _ => return false,
            }
        }

        match current_value {
            Some(serde_json::Value::Bool(b)) => *b,
            Some(serde_json::Value::Null) => true, // null means permission is enabled (default)
            _ => false,
        }
    }
}

/// Tool permission requirements
/// Maps MCP tool names to required Lago member permissions
pub struct ToolPermission {
    pub tool_name: &'static str,
    pub required_permission: &'static str,
}

/// All tool permission mappings
pub const TOOL_PERMISSIONS: &[ToolPermission] = &[
    // Invoice tools
    ToolPermission { tool_name: "get_invoice", required_permission: "invoices:view" },
    ToolPermission { tool_name: "list_invoices", required_permission: "invoices:view" },
    ToolPermission { tool_name: "preview_invoice", required_permission: "invoices:view" },
    ToolPermission { tool_name: "create_invoice", required_permission: "invoices:create" },
    ToolPermission { tool_name: "update_invoice", required_permission: "invoices:update" },
    ToolPermission { tool_name: "list_customer_invoices", required_permission: "invoices:view" },
    ToolPermission { tool_name: "refresh_invoice", required_permission: "draft_invoices:update" },
    ToolPermission { tool_name: "download_invoice", required_permission: "invoices:view" },
    ToolPermission { tool_name: "retry_invoice", required_permission: "invoices:update" },
    ToolPermission { tool_name: "retry_invoice_payment", required_permission: "invoices:update" },

    // Customer tools
    ToolPermission { tool_name: "get_customer", required_permission: "customers:view" },
    ToolPermission { tool_name: "list_customers", required_permission: "customers:view" },
    ToolPermission { tool_name: "create_customer", required_permission: "customers:create" },
    ToolPermission { tool_name: "get_customer_current_usage", required_permission: "customers:view" },

    // Subscription tools
    ToolPermission { tool_name: "list_subscriptions", required_permission: "subscriptions:view" },
    ToolPermission { tool_name: "get_subscription", required_permission: "subscriptions:view" },
    ToolPermission { tool_name: "list_customer_subscriptions", required_permission: "subscriptions:view" },
    ToolPermission { tool_name: "create_subscription", required_permission: "subscriptions:create" },
    ToolPermission { tool_name: "update_subscription", required_permission: "subscriptions:update" },
    ToolPermission { tool_name: "delete_subscription", required_permission: "subscriptions:update" },

    // Billable metric tools
    ToolPermission { tool_name: "get_billable_metric", required_permission: "billable_metrics:view" },
    ToolPermission { tool_name: "list_billable_metrics", required_permission: "billable_metrics:view" },
    ToolPermission { tool_name: "create_billable_metric", required_permission: "billable_metrics:create" },

    // Plan tools
    ToolPermission { tool_name: "list_plans", required_permission: "plans:view" },
    ToolPermission { tool_name: "get_plan", required_permission: "plans:view" },
    ToolPermission { tool_name: "create_plan", required_permission: "plans:create" },
    ToolPermission { tool_name: "update_plan", required_permission: "plans:update" },
    ToolPermission { tool_name: "delete_plan", required_permission: "plans:delete" },

    // Coupon tools
    ToolPermission { tool_name: "list_coupons", required_permission: "coupons:view" },
    ToolPermission { tool_name: "get_coupon", required_permission: "coupons:view" },
    ToolPermission { tool_name: "create_coupon", required_permission: "coupons:create" },
    ToolPermission { tool_name: "update_coupon", required_permission: "coupons:update" },
    ToolPermission { tool_name: "delete_coupon", required_permission: "coupons:delete" },

    // Applied coupon tools
    ToolPermission { tool_name: "list_applied_coupons", required_permission: "coupons:view" },
    ToolPermission { tool_name: "apply_coupon", required_permission: "coupons:attach" },

    // Credit note tools
    ToolPermission { tool_name: "list_credit_notes", required_permission: "credit_notes:view" },
    ToolPermission { tool_name: "get_credit_note", required_permission: "credit_notes:view" },
    ToolPermission { tool_name: "create_credit_note", required_permission: "credit_notes:create" },
    ToolPermission { tool_name: "update_credit_note", required_permission: "credit_notes:update" },

    // Event tools
    ToolPermission { tool_name: "get_event", required_permission: "developers:manage" },
    ToolPermission { tool_name: "create_event", required_permission: "developers:manage" },
    ToolPermission { tool_name: "list_events", required_permission: "developers:manage" },

    // Payment tools
    ToolPermission { tool_name: "list_payments", required_permission: "payments:view" },
    ToolPermission { tool_name: "get_payment", required_permission: "payments:view" },
    ToolPermission { tool_name: "list_customer_payments", required_permission: "payments:view" },
    ToolPermission { tool_name: "create_payment", required_permission: "payments:create" },

    // Activity log tools
    ToolPermission { tool_name: "get_activity_log", required_permission: "audit_logs:view" },
    ToolPermission { tool_name: "list_activity_logs", required_permission: "audit_logs:view" },

    // API log tools
    ToolPermission { tool_name: "get_api_log", required_permission: "developers:manage" },
    ToolPermission { tool_name: "list_api_logs", required_permission: "developers:manage" },
];

/// Extract member permissions from the request context
pub fn extract_member_permissions(
    context: &RequestContext<RoleServer>,
) -> Option<MemberPermissions> {
    context
        .extensions
        .get::<axum::http::request::Parts>()
        .and_then(|parts| {
            parts
                .headers
                .get("X-LAGO-MEMBER-PERMISSIONS")
                .and_then(|v| v.to_str().ok())
                .and_then(|encoded| {
                    BASE64
                        .decode(encoded)
                        .ok()
                        .and_then(|decoded| {
                            serde_json::from_slice(&decoded).ok()
                        })
                })
        })
}

/// Get the required permission for a tool
pub fn get_required_permission(tool_name: &str) -> Option<&'static str> {
    TOOL_PERMISSIONS
        .iter()
        .find(|p| p.tool_name == tool_name)
        .map(|p| p.required_permission)
}

/// Validate that the member has permission to use a specific tool
/// Returns Ok(()) if permitted, or Err(CallToolResult) with an error message
pub fn validate_tool_permission(
    context: &RequestContext<RoleServer>,
    tool_name: &str,
) -> Result<(), CallToolResult> {
    // Get required permission for this tool
    let required_permission = match get_required_permission(tool_name) {
        Some(p) => p,
        None => {
            // Tool not in permission mapping - allow by default
            tracing::warn!(tool_name, "Tool not found in permission mapping, allowing by default");
            return Ok(());
        }
    };

    // Extract member permissions from headers
    let member_permissions = match extract_member_permissions(context) {
        Some(p) => p,
        None => {
            // No permissions header - allow by default (backwards compatibility)
            tracing::debug!("No member permissions header found, allowing tool call");
            return Ok(());
        }
    };

    // Check if the member has the required permission
    if member_permissions.has_permission(required_permission) {
        Ok(())
    } else {
        tracing::info!(
            tool_name,
            required_permission,
            "Member lacks permission to use tool"
        );
        Err(error_result(format!(
            "Permission denied: You don't have the '{}' permission required to use this tool",
            required_permission
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_permission_simple() {
        let json = r#"{"customers": {"view": true, "create": false}}"#;
        let permissions: MemberPermissions = serde_json::from_str(json).unwrap();

        assert!(permissions.has_permission("customers:view"));
        assert!(!permissions.has_permission("customers:create"));
    }

    #[test]
    fn test_has_permission_nested() {
        let json = r#"{"customer_settings": {"update": {"tax_rates": true, "payment_terms": false}}}"#;
        let permissions: MemberPermissions = serde_json::from_str(json).unwrap();

        assert!(permissions.has_permission("customer_settings:update:tax_rates"));
        assert!(!permissions.has_permission("customer_settings:update:payment_terms"));
    }

    #[test]
    fn test_has_permission_null_means_true() {
        let json = r#"{"invoices": {"view": null}}"#;
        let permissions: MemberPermissions = serde_json::from_str(json).unwrap();

        assert!(permissions.has_permission("invoices:view"));
    }

    #[test]
    fn test_has_permission_missing() {
        let json = r#"{"customers": {"view": true}}"#;
        let permissions: MemberPermissions = serde_json::from_str(json).unwrap();

        assert!(!permissions.has_permission("invoices:view"));
        assert!(!permissions.has_permission("nonexistent:permission"));
    }

    #[test]
    fn test_get_required_permission() {
        assert_eq!(get_required_permission("get_invoice"), Some("invoices:view"));
        assert_eq!(get_required_permission("create_customer"), Some("customers:create"));
        assert_eq!(get_required_permission("unknown_tool"), None);
    }
}

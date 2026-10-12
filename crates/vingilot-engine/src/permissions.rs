use std::collections::HashMap;

/// Types of browser permissions requested by websites.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PermissionType {
    Geolocation,
    Notifications,
    Microphone,
    Camera,
    ClipboardRead,
}

/// User or policy decision for a permission request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionDecision {
    Allow,
    Deny,
}

/// Permission prompt dialog state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionPrompt {
    pub origin: String, // e.g. "https://example.com"
    pub permission_type: PermissionType,
}

/// Permission manager tracking per-domain remembered decisions and active prompts.
#[derive(Debug, Clone, Default)]
pub struct PermissionManager {
    remembered_decisions: HashMap<(String, PermissionType), PermissionDecision>,
    active_prompt: Option<PermissionPrompt>,
}

impl PermissionManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates a permission request: returns cached decision if known, or creates an active prompt.
    pub fn request_permission(
        &mut self,
        origin: &str,
        permission_type: PermissionType,
    ) -> Option<PermissionDecision> {
        let key = (origin.to_string(), permission_type);
        if let Some(decision) = self.remembered_decisions.get(&key) {
            Some(*decision)
        } else {
            self.active_prompt = Some(PermissionPrompt {
                origin: origin.to_string(),
                permission_type,
            });
            None
        }
    }

    pub fn active_prompt(&self) -> Option<&PermissionPrompt> {
        self.active_prompt.as_ref()
    }

    /// Resolves the active permission prompt.
    pub fn respond_to_active_prompt(
        &mut self,
        decision: PermissionDecision,
        remember: bool,
    ) -> Option<PermissionDecision> {
        let prompt = self.active_prompt.take()?;
        if remember {
            self.remembered_decisions
                .insert((prompt.origin, prompt.permission_type), decision);
        }
        Some(decision)
    }

    /// Clears remembered permissions for an origin.
    pub fn clear_for_origin(&mut self, origin: &str) {
        self.remembered_decisions.retain(|(o, _), _| o != origin);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_request_and_remember() {
        let mut pm = PermissionManager::new();

        // First request: unknown, prompt created
        let result = pm.request_permission("https://maps.example.com", PermissionType::Geolocation);
        assert!(result.is_none());
        assert!(pm.active_prompt().is_some());

        // Respond with Allow and remember
        pm.respond_to_active_prompt(PermissionDecision::Allow, true);
        assert!(pm.active_prompt().is_none());

        // Subsequent request: automatically allowed
        let cached = pm.request_permission("https://maps.example.com", PermissionType::Geolocation);
        assert_eq!(cached, Some(PermissionDecision::Allow));

        // Different permission on same domain requires new prompt
        let cam = pm.request_permission("https://maps.example.com", PermissionType::Camera);
        assert!(cam.is_none());
    }
}

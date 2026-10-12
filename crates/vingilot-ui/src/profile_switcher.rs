use uuid::Uuid;
use vingilot_core::models::Profile;

/// UI panel and switcher for active browsing profiles.
#[derive(Debug, Clone)]
pub struct ProfileSwitcher {
    profiles: Vec<Profile>,
    active_profile_id: Option<Uuid>,
}

impl Default for ProfileSwitcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfileSwitcher {
    pub fn new() -> Self {
        Self {
            profiles: Vec::new(),
            active_profile_id: None,
        }
    }

    pub fn set_profiles(&mut self, profiles: Vec<Profile>) {
        if self.active_profile_id.is_none() && !profiles.is_empty() {
            self.active_profile_id = Some(profiles[0].id);
        }
        self.profiles = profiles;
    }

    pub fn profiles(&self) -> &[Profile] {
        &self.profiles
    }

    pub fn active_profile(&self) -> Option<&Profile> {
        self.active_profile_id
            .and_then(|id| self.profiles.iter().find(|p| p.id == id))
    }

    pub fn select_profile(&mut self, id: Uuid) -> bool {
        if self.profiles.iter().any(|p| p.id == id) {
            self.active_profile_id = Some(id);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use vingilot_core::models::ProfileType;

    #[test]
    fn test_profile_switcher_selection() {
        let mut switcher = ProfileSwitcher::new();
        let p1 = Profile {
            id: Uuid::new_v4(),
            name: "Standard".to_string(),
            profile_type: ProfileType::Standard,
            color_or_icon: None,
            data_dir: "/tmp/standard".to_string(),
            default_search_engine_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let p2 = Profile {
            id: Uuid::new_v4(),
            name: "Work".to_string(),
            profile_type: ProfileType::Work,
            color_or_icon: None,
            data_dir: "/tmp/work".to_string(),
            default_search_engine_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        switcher.set_profiles(vec![p1.clone(), p2.clone()]);
        assert_eq!(switcher.active_profile().unwrap().id, p1.id);

        assert!(switcher.select_profile(p2.id));
        assert_eq!(switcher.active_profile().unwrap().name, "Work");
    }
}

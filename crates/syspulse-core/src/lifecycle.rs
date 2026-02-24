use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Failed,
    Scheduled,
}

impl LifecycleState {
    pub fn can_transition_to(&self, target: LifecycleState) -> bool {
        use LifecycleState::*;
        matches!(
            (self, target),
            (Stopped, Starting)
                | (Stopped, Scheduled)
                | (Starting, Running)
                | (Starting, Failed)
                | (Starting, Stopping)
                | (Running, Stopping)
                | (Running, Failed)
                | (Stopping, Stopped)
                | (Stopping, Failed)
                | (Failed, Starting)
                | (Failed, Stopped)
                | (Scheduled, Starting)
                | (Scheduled, Stopped)
        )
    }

    pub fn transition_to(&self, target: LifecycleState) -> crate::error::Result<LifecycleState> {
        if self.can_transition_to(target) {
            Ok(target)
        } else {
            Err(crate::error::SyspulseError::InvalidStateTransition {
                from: format!("{:?}", self),
                to: format!("{:?}", target),
            })
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Self::Starting | Self::Running | Self::Stopping)
    }
}

impl std::fmt::Display for LifecycleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stopped => write!(f, "stopped"),
            Self::Starting => write!(f, "starting"),
            Self::Running => write!(f, "running"),
            Self::Stopping => write!(f, "stopping"),
            Self::Failed => write!(f, "failed"),
            Self::Scheduled => write!(f, "scheduled"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use LifecycleState::*;

    #[test]
    fn valid_transitions() {
        let valid = vec![
            (Stopped, Starting),
            (Stopped, Scheduled),
            (Starting, Running),
            (Starting, Failed),
            (Starting, Stopping),
            (Running, Stopping),
            (Running, Failed),
            (Stopping, Stopped),
            (Stopping, Failed),
            (Failed, Starting),
            (Failed, Stopped),
            (Scheduled, Starting),
            (Scheduled, Stopped),
        ];

        for (from, to) in valid {
            assert!(
                from.can_transition_to(to),
                "{:?} -> {:?} should be valid",
                from,
                to
            );
            assert!(
                from.transition_to(to).is_ok(),
                "{:?} -> {:?} transition_to should succeed",
                from,
                to
            );
        }
    }

    #[test]
    fn invalid_transitions() {
        let invalid = vec![
            (Stopped, Running),
            (Stopped, Stopping),
            (Stopped, Failed),
            (Starting, Stopped),
            (Starting, Scheduled),
            (Running, Starting),
            (Running, Stopped),
            (Running, Scheduled),
            (Stopping, Starting),
            (Stopping, Running),
            (Stopping, Scheduled),
            (Failed, Running),
            (Failed, Stopping),
            (Failed, Scheduled),
            (Scheduled, Running),
            (Scheduled, Stopping),
            (Scheduled, Failed),
        ];

        for (from, to) in invalid {
            assert!(
                !from.can_transition_to(to),
                "{:?} -> {:?} should be invalid",
                from,
                to
            );
            assert!(
                from.transition_to(to).is_err(),
                "{:?} -> {:?} transition_to should fail",
                from,
                to
            );
        }
    }

    #[test]
    fn self_transitions_are_invalid() {
        let all_states = vec![Stopped, Starting, Running, Stopping, Failed, Scheduled];
        for state in all_states {
            assert!(
                !state.can_transition_to(state),
                "{:?} -> {:?} self-transition should be invalid",
                state,
                state
            );
        }
    }

    #[test]
    fn is_active() {
        assert!(!Stopped.is_active());
        assert!(Starting.is_active());
        assert!(Running.is_active());
        assert!(Stopping.is_active());
        assert!(!Failed.is_active());
        assert!(!Scheduled.is_active());
    }

    #[test]
    fn display() {
        assert_eq!(Stopped.to_string(), "stopped");
        assert_eq!(Starting.to_string(), "starting");
        assert_eq!(Running.to_string(), "running");
        assert_eq!(Stopping.to_string(), "stopping");
        assert_eq!(Failed.to_string(), "failed");
        assert_eq!(Scheduled.to_string(), "scheduled");
    }
}

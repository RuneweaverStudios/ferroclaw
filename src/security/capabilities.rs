//! Capability-based permission system.
//!
//! Each tool declares required capabilities. A session has a capability set.
//! Tool execution is blocked if the session lacks the required capabilities.
//! This prevents the OpenClaw problem of granting full access by default.

use crate::types::{Capability, CapabilitySet};

/// Create a capability set from config values.
pub fn capabilities_from_config(caps: &[Capability]) -> CapabilitySet {
    CapabilitySet::new(caps.iter().copied())
}

/// Display capabilities as a readable list.
pub fn format_capabilities(caps: &CapabilitySet) -> String {
    let mut sorted: Vec<String> = caps.capabilities.iter().map(|c| c.to_string()).collect();
    sorted.sort();
    sorted.join(", ")
}

/// Check if a set of required capabilities is satisfied and return
/// a user-friendly error message if not.
pub fn check_with_message(
    session_caps: &CapabilitySet,
    required: &[Capability],
    tool_name: &str,
) -> Result<(), String> {
    for cap in required {
        if !session_caps.has(*cap) {
            return Err(format!(
                "Tool '{tool_name}' requires capability '{cap}' which is not granted. \
                 Current capabilities: [{}]. \
                 Add '{cap}' to security.default_capabilities in config.toml to allow.",
                format_capabilities(session_caps)
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capabilities_from_config() {
        let caps = capabilities_from_config(&[Capability::FsRead, Capability::NetOutbound]);
        assert!(caps.has(Capability::FsRead));
        assert!(caps.has(Capability::NetOutbound));
        assert!(!caps.has(Capability::FsWrite));
    }

    #[test]
    fn test_check_with_message_ok() {
        let caps = CapabilitySet::new([Capability::FsRead]);
        assert!(check_with_message(&caps, &[Capability::FsRead], "read_file").is_ok());
    }

    #[test]
    fn test_check_with_message_denied() {
        let caps = CapabilitySet::new([Capability::FsRead]);
        let result = check_with_message(&caps, &[Capability::FsWrite], "write_file");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("fs_write"));
    }

    #[test]
    fn test_format_capabilities() {
        let caps = CapabilitySet::new([Capability::FsRead, Capability::NetOutbound]);
        let formatted = format_capabilities(&caps);
        assert!(formatted.contains("fs_read"));
        assert!(formatted.contains("net_outbound"));
    }
}

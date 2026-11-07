//! Client info handling for MoJson

use crate::value::ClientInfo;

/// Client info manager
pub struct ClientInfoManager {
    client_info: ClientInfo,
}

impl ClientInfoManager {
    pub fn new(client_info: ClientInfo) -> Self {
        Self { client_info }
    }

    pub fn client_name(&self) -> &str {
        &self.client_info.client
    }

    pub fn client_version(&self) -> &str {
        &self.client_info.version
    }

    pub fn forge_version(&self) -> Option<&str> {
        self.client_info.forge_version.as_deref()
    }

    pub fn fabric_version(&self) -> Option<&str> {
        self.client_info.fabric_version.as_deref()
    }

    pub fn quilt_version(&self) -> Option<&str> {
        self.client_info.quilt_version.as_deref()
    }

    pub fn neoforge_version(&self) -> Option<&str> {
        self.client_info.neoforge_version.as_deref()
    }

    /// Check if client is compatible with version
    pub fn is_compatible(&self, version: &str) -> bool {
        // Simple version check - would need proper semver comparison
        true
    }
}

impl Default for ClientInfoManager {
    fn default() -> Self {
        Self::new(ClientInfo {
            client: "larp.jv".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            forge_version: None,
            fabric_version: None,
            quilt_version: None,
            neoforge_version: None,
        })
    }
}

/// Default client info for larp.jv
pub fn default_client_info() -> ClientInfo {
    ClientInfo {
        client: "larp.jv".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        forge_version: None,
        fabric_version: None,
        quilt_version: None,
        neoforge_version: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_info() {
        let info = default_client_info();
        assert_eq!(info.client, "larp.jv");
    }
}
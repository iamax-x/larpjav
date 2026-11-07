//! Core value types and data models for larp.jv
//!
//! These types mirror the Mojang launcher JSON structures and are used
//! throughout the launcher for version manifests, libraries, rules, etc.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::error::{LarpError, LarpResult};

/// Authentication type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthType {
    Microsoft,
    ElyBy,
    Offline,
}

impl std::fmt::Display for AuthType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthType::Microsoft => write!(f, "microsoft"),
            AuthType::ElyBy => write!(f, "elyby"),
            AuthType::Offline => write!(f, "offline"),
        }
    }
}

impl std::str::FromStr for AuthType {
    type Err = LarpError;
    fn from_str(s: &str) -> LarpResult<Self> {
        match s.to_lowercase().as_str() {
            "microsoft" => Ok(AuthType::Microsoft),
            "elyby" | "ely.by" => Ok(AuthType::ElyBy),
            "offline" => Ok(AuthType::Offline),
            _ => Err(LarpError::Auth(crate::error::AuthError::UnsupportedType(s.to_string()))),
        }
    }
}

/// Minecraft account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,                    // UUID
    pub username: String,
    pub auth_type: AuthType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,  // Encrypted at rest
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>, // Encrypted at rest
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,          // Player UUID (different from account ID for Microsoft)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skin_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cape_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skin_model: Option<String>,    // "classic" or "slim"
    pub selected: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used: chrono::DateTime<chrono::Utc>,
}

impl Account {
    /// Create a new offline account
    pub fn offline(username: impl Into<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            username: username.into(),
            auth_type: AuthType::Offline,
            access_token: None,
            refresh_token: None,
            expires_at: None,
            uuid: None,
            skin_url: None,
            cape_url: None,
            skin_model: None,
            selected: false,
            created_at: now,
            last_used: now,
        }
    }

    /// Check if account needs token refresh
    pub fn needs_refresh(&self) -> bool {
        match (self.auth_type, self.expires_at) {
            (AuthType::Offline, _) => false,
            (_, Some(expires)) => chrono::Utc::now() >= expires - chrono::Duration::minutes(5),
            (_, None) => true,
        }
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        match (self.auth_type, self.expires_at) {
            (AuthType::Offline, _) => false,
            (_, Some(expires)) => chrono::Utc::now() >= expires,
            (_, None) => true,
        }
    }
}

/// Account manager for storing/retrieving accounts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountManager {
    #[serde(default)]
    pub accounts: Vec<Account>,
    #[serde(skip)]
    encryption_key: Option<[u8; 32]>,
}

impl AccountManager {
    pub fn new() -> Self {
        Self {
            accounts: Vec::new(),
            encryption_key: None,
        }
    }

    pub fn with_encryption_key(mut self, key: [u8; 32]) -> Self {
        self.encryption_key = Some(key);
        self
    }

    pub fn add_account(&mut self, account: Account) {
        // Deselect all others if this one is selected
        if account.selected {
            for acc in &mut self.accounts {
                acc.selected = false;
            }
        }
        self.accounts.push(account);
    }

    pub fn remove_account(&mut self, id: &str) -> bool {
        let len = self.accounts.len();
        self.accounts.retain(|a| a.id != id);
        self.accounts.len() != len
    }

    pub fn get_account(&self, id: &str) -> Option<&Account> {
        self.accounts.iter().find(|a| a.id == id)
    }

    pub fn get_selected(&self) -> Option<&Account> {
        self.accounts.iter().find(|a| a.selected)
    }

    pub fn select_account(&mut self, id: &str) -> bool {
        for acc in &mut self.accounts {
            acc.selected = acc.id == id;
        }
        self.accounts.iter().any(|a| a.selected && a.id == id)
    }

    pub fn list_accounts(&self) -> &[Account] {
        &self.accounts
    }
}

impl Default for AccountManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Mojang version manifest (version_manifest.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<VersionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: VersionType,
    pub url: String,
    pub time: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "releaseTime")]
    pub release_time: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub sha1: Option<String>,
    #[serde(default)]
    pub compliance_level: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VersionType {
    Release,
    Snapshot,
    OldBeta,
    OldAlpha,
}

/// Complete version JSON (downloaded from version URL)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    #[serde(rename = "assetIndex")]
    pub asset_index: AssetIndex,
    pub assets: String,
    #[serde(default)]
    pub compliance_level: Option<u32>,
    #[serde(default)]
    pub download: Option<DownloadInfo>,
    pub id: String,
    pub libraries: Vec<Library>,
    #[serde(default)]
    pub logging: Option<Logging>,
    #[serde(default)]
    pub main_class: String,
    #[serde(default)]
    #[serde(rename = "minimumLauncherVersion")]
    pub minimum_launcher_version: Option<u32>,
    #[serde(default)]
    pub arguments: Arguments,
    #[serde(default)]
    #[serde(rename = "minecraftArguments")]
    pub minecraft_arguments: Option<String>,
    #[serde(default)]
    #[serde(rename = "releaseTime")]
    pub release_time: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    #[serde(rename = "time")]
    pub time: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "type")]
    pub version_type: String,
    #[serde(default)]
    pub inherits_from: Option<String>,
    #[serde(default)]
    #[serde(rename = "jar")]
    pub jar_url: Option<String>,
    #[serde(default)]
    pub sha1: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetIndex {
    pub id: String,
    pub url: String,
    #[serde(rename = "totalSize")]
    pub total_size: u64,
    pub sha1: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadInfo {
    pub url: String,
    pub sha1: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Logging {
    pub client: LoggingClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingClient {
    pub file: LoggingFile,
    #[serde(default)]
    pub argument: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingFile {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

/// Game arguments (modern format)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Arguments {
    #[serde(default)]
    pub game: Vec<ArgumentValue>,
    #[serde(default)]
    pub jvm: Vec<ArgumentValue>,
}

/// Argument value (string or rule-based object)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArgumentValue {
    Simple(String),
    Complex(ArgumentRule),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgumentRule {
    pub rules: Vec<Rule>,
    pub value: Vec<String>,
}

/// Rule for OS/feature-based conditional logic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub action: RuleAction,
    #[serde(default)]
    pub os: Option<OsRule>,
    #[serde(default)]
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "allow")]
pub enum RuleAction {
    Allow,
    Disallow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsRule {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub arch: Option<String>,
}

/// Library artifact (from version JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub name: String,  // Maven coordinate: group:artifact:version
    #[serde(default)]
    pub downloads: Option<LibraryDownloads>,
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub natives: Option<HashMap<String, String>>,  // classifier per OS
    #[serde(default)]
    #[serde(rename = "extract")]
    pub extract_rules: Option<ExtractRules>,
    #[serde(default)]
    #[serde(rename = "insert")]
    pub insert_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryDownloads {
    pub artifact: Option<ArtifactInfo>,
    #[serde(default)]
    pub classifiers: HashMap<String, ArtifactInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactInfo {
    pub url: String,
    pub sha1: String,
    pub size: u64,
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractRules {
    #[serde(default)]
    pub exclude: Vec<String>,
}

/// Library substitution (from substitutions.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibrarySubstitution {
    pub name: String,           // Original maven coordinate
    pub target: String,         // Replacement maven coordinate
    #[serde(default)]
    pub rules: Vec<Rule>,
}

/// Substitution map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubstitutionMap {
    pub libraries: Vec<LibrarySubstitution>,
}

/// Client info (from client.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub client: String,
    pub version: String,
    #[serde(default)]
    pub forge_version: Option<String>,
    #[serde(default)]
    pub fabric_version: Option<String>,
    #[serde(default)]
    pub quilt_version: Option<String>,
    #[serde(default)]
    pub neoforge_version: Option<String>,
}

/// Dependent library (for native extraction)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependentLibrary {
    pub name: String,
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub natives: Option<HashMap<String, String>>,
    #[serde(default)]
    #[serde(rename = "extract")]
    pub extract_rules: Option<ExtractRules>,
}

/// Native library extractable interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeLibraryExtractable {
    pub name: String,
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub natives: Option<HashMap<String, String>>,
    #[serde(default)]
    #[serde(rename = "extract")]
    pub extract_rules: Option<ExtractRules>,
}

/// Extract settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractSettings {
    #[serde(default)]
    pub exclude: Vec<String>,
}

/// MoJson rule (alias for Rule)
pub type MoJsonRule = Rule;

/// Instance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub version_id: String,
    pub game_dir: String,
    #[serde(default)]
    pub java_path: Option<String>,
    #[serde(default)]
    pub jvm_args: Vec<String>,
    #[serde(default)]
    pub game_args: Vec<String>,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,  // Base64 encoded PNG
    #[serde(default)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub last_played: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub play_time: u64,  // milliseconds
    #[serde(default)]
    pub custom_resolution: Option<Resolution>,
    #[serde(default)]
    pub mods: Vec<ModInfo>,
    #[serde(default)]
    pub resource_packs: Vec<String>,
    #[serde(default)]
    pub shader_pack: Option<String>,
    #[serde(default)]
    pub environment: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInfo {
    pub id: String,
    pub version: String,
    pub loader: ModLoaderType,
    pub enabled: bool,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub sha1: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModLoaderType {
    Forge,
    Fabric,
    Quilt,
    NeoForge,
    Optifine,
    LegacyFabric,
    BTA,
    Vanilla,
    Custom(String),
}

/// Mod loader metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModLoaderMetadata {
    pub loader: ModLoaderType,
    pub version: String,
    pub mc_version: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub sha1: Option<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

impl Library {
    /// Parse maven coordinate into (group, artifact, version)
    pub fn parse_maven(&self) -> LarpResult<(String, String, String)> {
        let parts: Vec<&str> = self.name.split(':').collect();
        if parts.len() != 3 {
            return Err(LarpError::MoJson(crate::error::MoJsonError::InvalidLibraryFormat(
                self.name.clone(),
            )));
        }
        Ok((parts[0].to_string(), parts[1].to_string(), parts[2].to_string()))
    }

    /// Get classifier for current OS
    pub fn classifier_for_os(&self, os: &str) -> Option<&String> {
        self.natives.as_ref()?.get(os)
    }

    /// Check if library should be included for current OS/features
    pub fn matches(&self, os: &str, arch: &str, features: &HashMap<String, bool>) -> bool {
        crate::mojson::rules::evaluate_rules(&self.rules, os, arch, features)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_offline() {
        let acc = Account::offline("TestPlayer");
        assert_eq!(acc.auth_type, AuthType::Offline);
        assert!(!acc.needs_refresh());
        assert!(!acc.is_expired());
    }

    #[test]
    fn test_library_parse_maven() {
        let lib = Library {
            name: "com.mojang:authlib:1.0.0".to_string(),
            downloads: None,
            rules: vec![],
            natives: None,
            extract_rules: None,
            insert_at: None,
        };
        let (g, a, v) = lib.parse_maven().unwrap();
        assert_eq!(g, "com.mojang");
        assert_eq!(a, "authlib");
        assert_eq!(v, "1.0.0");
    }

    #[test]
    fn test_version_manifest_deserialize() {
        let json = r#"{
            "latest": {"release": "1.20.4", "snapshot": "24w03a"},
            "versions": [
                {"id": "1.20.4", "type": "release", "url": "https://example.com/1.20.4.json", "time": "2023-12-07T14:23:00+00:00", "releaseTime": "2023-12-07T14:23:00+00:00"}
            ]
        }"#;
        let manifest: VersionManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.latest.release, "1.20.4");
        assert_eq!(manifest.versions.len(), 1);
    }
}
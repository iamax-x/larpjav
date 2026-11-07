//! Error types for larp-jv-core

use std::fmt;
use thiserror::Error;

/// Result type alias for larp-jv-core
pub type LarpResult<T> = Result<T, LarpError>;

/// Main error enum for the larp.jv launcher
#[derive(Debug, Error)]
pub enum LarpError {
    /// Authentication errors
    #[error("Authentication failed: {0}")]
    Auth(#[from] AuthError),

    /// Download errors
    #[error("Download failed: {0}")]
    Download(#[from] DownloadError),

    /// JRE/JVM errors
    #[error("JRE error: {0}")]
    Jre(#[from] JreError),

    /// Mod loader/installer errors
    #[error("Modding error: {0}")]
    Modding(#[from] ModdingError),

    /// Instance management errors
    #[error("Instance error: {0}")]
    Instance(#[from] InstanceError),

    /// Mojang JSON parsing/validation errors
    #[error("MoJson error: {0}")]
    MoJson(#[from] MoJsonError),

    /// I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    /// TOML errors
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    /// TOML serialization errors
    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    /// ZIP archive errors
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// HTTP request errors
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// URL parsing errors
    #[error("URL error: {0}")]
    Url(#[from] url::ParseError),

    /// Hex decoding errors
    #[error("Hex error: {0}")]
    Hex(#[from] hex::FromHexError),

    /// Base64 decoding errors
    #[error("Base64 error: {0}")]
    Base64(#[from] base64::DecodeError),

    /// Crypto errors
    #[error("Crypto error: {0}")]
    Crypto(#[from] ring::error::Unspecified),

    /// AES-GCM errors
    #[error("AES-GCM error: {0}")]
    AesGcm(#[from] aes_gcm::Error),

    /// UTF-8 conversion errors
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    /// String conversion errors
    #[error("String error: {0}")]
    String(#[from] std::string::FromUtf8Error),

    /// NUL byte errors
    #[error("NUL byte error: {0}")]
    Nul(#[from] std::ffi::NulError),

    /// Time parsing errors
    #[error("Time error: {0}")]
    Time(#[from] chrono::ParseError),

    /// UUID errors
    #[error("UUID error: {0}")]
    Uuid(#[from] uuid::Error),

    /// Semver errors
    #[error("Semver error: {0}")]
    Semver(#[from] semver::Error),

    /// Walkdir errors
    #[error("Walkdir error: {0}")]
    Walkdir(#[from] walkdir::Error),

    /// Glob errors
    #[error("Glob error: {0}")]
    Glob(#[from] glob::GlobError),

    /// Generic error with context
    #[error("{0}")]
    Other(String),

    /// Multiple errors aggregated
    #[error("Multiple errors: {0:?}")]
    Multiple(Vec<LarpError>),
}

/// Authentication-specific errors
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Account already exists: {0}")]
    AccountExists(String),

    #[error("OAuth2 error: {0}")]
    OAuth2(String),

    #[error("Microsoft auth error: {0}")]
    Microsoft(String),

    #[error("Ely.by auth error: {0}")]
    ElyBy(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Token refresh failed: {0}")]
    TokenRefreshFailed(String),

    #[error("Network error during auth: {0}")]
    Network(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Crypto error: {0}")]
    Crypto(#[from] ring::error::Unspecified),

    #[error("AES-GCM error: {0}")]
    AesGcm(#[from] aes_gcm::Error),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("User cancelled")]
    UserCancelled,

    #[error("Unsupported auth type: {0}")]
    UnsupportedType(String),
}

/// Download-specific errors
#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("HTTP {status}: {message}")]
    HttpStatus { status: u16, message: String },

    #[error("Hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },

    #[error("File size mismatch: expected {expected}, got {actual}")]
    SizeMismatch { expected: u64, actual: u64 },

    #[error("Resume not supported by server")]
    ResumeNotSupported,

    #[error("Partial download cannot be resumed: {0}")]
    CannotResume(String),

    #[error("Destination exists and overwrite is false")]
    DestinationExists,

    #[error("Insufficient disk space: need {needed}, have {available}")]
    InsufficientSpace { needed: u64, available: u64 },

    #[error("Timeout after {seconds}s")]
    Timeout { seconds: u64 },

    #[error("Too many redirects")]
    TooManyRedirects,

    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Task cancelled")]
    Cancelled,

    #[error("Verification failed: {0}")]
    VerificationFailed(String),
}

/// JRE/JVM errors
#[derive(Debug, Error)]
pub enum JreError {
    #[error("No suitable JRE found for version {version}")]
    NotFound { version: String },

    #[error("JRE download failed: {0}")]
    DownloadFailed(#[from] DownloadError),

    #[error("JRE extraction failed: {0}")]
    ExtractionFailed(String),

    #[error("Invalid JRE structure: {0}")]
    InvalidStructure(String),

    #[error("Java version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: String, found: String },

    #[error("Class file parsing error: {0}")]
    ClassFileParse(String),

    #[error("Class version unsupported: {version}")]
    UnsupportedClassVersion { version: u16 },

    #[error("JVM launch failed: {0}")]
    LaunchFailed(String),

    #[error("JVM process exited with code {code}: {stderr}")]
    ProcessExited { code: i32, stderr: String },

    #[error("Native library not found: {lib}")]
    NativeLibraryMissing { lib: String },

    #[error("Native library load failed: {lib}: {error}")]
    NativeLibraryLoadFailed { lib: String, error: String },

    #[error("Memory allocation failed: {0}")]
    MemoryAllocationFailed(String),

    #[error("Invalid JVM argument: {0}")]
    InvalidArgument(String),
}

/// Modding errors
#[derive(Debug, Error)]
pub enum ModdingError {
    #[error("Mod loader not found: {loader}")]
    LoaderNotFound { loader: String },

    #[error("Mod loader version not found: {loader} {version}")]
    LoaderVersionNotFound { loader: String, version: String },

    #[error("Forge installer error: {0}")]
    Forge(String),

    #[error("Fabric installer error: {0}")]
    Fabric(String),

    #[error("Quilt installer error: {0}")]
    Quilt(String),

    #[error("NeoForge installer error: {0}")]
    NeoForge(String),

    #[error("OptiFine error: {0}")]
    OptiFine(String),

    #[error("Legacy Fabric error: {0}")]
    LegacyFabric(String),

    #[error("Modpack (MRPACK) error: {0}")]
    Modpack(String),

    #[error("Mod not found: {mod_id}")]
    ModNotFound { mod_id: String },

    #[error("Mod version not found: {mod_id} {version}")]
    ModVersionNotFound { mod_id: String, version: String },

    #[error("Dependency resolution failed: {0}")]
    DependencyResolution(String),

    #[error("Incompatible mods: {mods:?}")]
    IncompatibleMods { mods: Vec<String> },

    #[error("Installation failed: {0}")]
    InstallationFailed(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Instance management errors
#[derive(Debug, Error)]
pub enum InstanceError {
    #[error("Instance not found: {0}")]
    NotFound(String),

    #[error("Instance already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid instance name: {0}")]
    InvalidName(String),

    #[error("Instance directory not found: {0}")]
    DirectoryNotFound(String),

    #[error("Instance config parse error: {0}")]
    ConfigParse(#[from] toml::de::Error),

    #[error("Instance config serialize error: {0}")]
    ConfigSerialize(#[from] toml::ser::Error),

    #[error("Version not found for instance: {0}")]
    VersionNotFound(String),

    #[error("Game directory not accessible: {0}")]
    GameDirInaccessible(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// MoJson parsing/validation errors
#[derive(Debug, Error)]
pub enum MoJsonError {
    #[error("Invalid version manifest: {0}")]
    InvalidManifest(String),

    #[error("Rule evaluation failed: {rule} - {reason}")]
    RuleEvaluation { rule: String, reason: String },

    #[error("Library not found: {group}:{artifact}:{version}")]
    LibraryNotFound { group: String, artifact: String, version: String },

    #[error("Library download failed: {0}")]
    LibraryDownload(#[from] DownloadError),

    #[error("Native library extraction failed: {0}")]
    NativeExtraction(String),

    #[error("Substitution not found: {key}")]
    SubstitutionNotFound { key: String },

    #[error("Mirror tampered: {mirror}")]
    MirrorTampered { mirror: String },

    #[error("All mirrors failed")]
    AllMirrorsFailed,

    #[error("Invalid library format: {0}")]
    InvalidLibraryFormat(String),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl LarpError {
    /// Create a new "other" error from a string
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }

    /// Create a new "other" error from a format string
    pub fn other_fmt(args: fmt::Arguments) -> Self {
        Self::Other(args.to_string())
    }

    /// Check if this error is recoverable (e.g., network timeout vs auth failure)
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            LarpError::Download(DownloadError::Timeout { .. })
                | LarpError::Download(DownloadError::Network(_))
                | LarpError::Download(DownloadError::HttpStatus { status: 500..=599, .. })
                | LarpError::Io(_)
                | LarpError::Http(_)
        )
    }

    /// Get a user-friendly error code for telemetry/UI
    pub fn error_code(&self) -> &'static str {
        match self {
            LarpError::Auth(_) => "AUTH_FAILED",
            LarpError::Download(_) => "DOWNLOAD_FAILED",
            LarpError::Jre(_) => "JRE_ERROR",
            LarpError::Modding(_) => "MODDING_ERROR",
            LarpError::Instance(_) => "INSTANCE_ERROR",
            LarpError::MoJson(_) => "MOJSON_ERROR",
            LarpError::Io(_) => "IO_ERROR",
            LarpError::Serde(_) => "SERIALIZATION_ERROR",
            LarpError::Toml(_) | LarpError::TomlSer(_) => "TOML_ERROR",
            LarpError::Zip(_) => "ZIP_ERROR",
            LarpError::Http(_) => "HTTP_ERROR",
            LarpError::Crypto(_) | LarpError::AesGcm(_) => "CRYPTO_ERROR",
            LarpError::Other(_) => "UNKNOWN_ERROR",
            LarpError::Multiple(_) => "MULTIPLE_ERRORS",
            _ => "INTERNAL_ERROR",
        }
    }
}

impl From<Vec<LarpError>> for LarpError {
    fn from(errors: Vec<LarpError>) -> Self {
        if errors.len() == 1 {
            errors.into_iter().next().unwrap()
        } else {
            LarpError::Multiple(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        let err = LarpError::Auth(AuthError::InvalidCredentials);
        assert_eq!(err.error_code(), "AUTH_FAILED");

        let err = LarpError::Download(DownloadError::Timeout { seconds: 30 });
        assert_eq!(err.error_code(), "DOWNLOAD_FAILED");
        assert!(err.is_recoverable());

        let err = LarpError::Auth(AuthError::InvalidCredentials);
        assert!(!err.is_recoverable());
    }

    #[test]
    fn test_multiple_errors() {
        let errs = vec![
            LarpError::Download(DownloadError::Timeout { seconds: 30 }),
            LarpError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "file")),
        ];
        let combined: LarpError = errs.into();
        assert!(matches!(combined, LarpError::Multiple(_)));
    }
}
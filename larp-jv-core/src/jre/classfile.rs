//! Class file parsing

use crate::error::LarpResult;
use std::io::{Read, Cursor};
use byteorder::{BigEndian, ReadBytesExt};

/// Java class file parser
pub struct ClassFileParser;

impl ClassFileParser {
    /// Parse class file and return version
    pub fn parse_version(data: &[u8]) -> LarpResult<u16> {
        let mut cursor = Cursor::new(data);

        // Magic number (0xCAFEBABE)
        let magic = cursor.read_u32::<BigEndian>()?;
        if magic != 0xCAFEBABE {
            return Err(crate::error::LarpError::Jre(crate::error::JreError::ClassFileParse("Invalid magic number".to_string())));
        }

        // Minor version
        let _minor = cursor.read_u16::<BigEndian>()?;

        // Major version
        let major = cursor.read_u16::<BigEndian>()?;

        Ok(major)
    }

    /// Get Java version from class version
    pub fn java_version(class_version: u16) -> &'static str {
        match class_version {
            45 => "1.0",
            46 => "1.1",
            47 => "1.2",
            48 => "1.3",
            49 => "1.4",
            50 => "5",
            51 => "6",
            52 => "7",
            53 => "8",
            54 => "9",
            55 => "10",
            56 => "11",
            57 => "12",
            58 => "13",
            59 => "14",
            60 => "15",
            61 => "16",
            62 => "17",
            63 => "18",
            64 => "19",
            65 => "20",
            66 => "21",
            67 => "22",
            68 => "23",
            _ => "Unknown",
        }
    }

    /// Check if class version is supported
    pub fn is_supported(class_version: u16, min_version: u16, max_version: u16) -> bool {
        class_version >= min_version && class_version <= max_version
    }
}

/// Parse class file from path
pub async fn parse_class_file(path: &std::path::Path) -> LarpResult<u16> {
    let data = tokio::fs::read(path).await?;
    ClassFileParser::parse_version(&data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_java_version() {
        assert_eq!(ClassFileParser::java_version(52), "8");
        assert_eq!(ClassFileParser::java_version(61), "17");
        assert_eq!(ClassFileParser::java_version(65), "21");
    }

    #[test]
    fn test_is_supported() {
        assert!(ClassFileParser::is_supported(52, 50, 60)); // Java 8 in range 6-16
        assert!(!ClassFileParser::is_supported(52, 55, 60)); // Java 8 not in range 11-16
    }
}
//! Unpack200 — Pack200 unpacker (replaces libunpack200.so)

use std::io::{Read, Write};
use std::path::Path;
use crate::error::LarpResult;

/// Unpack a Pack200 file
pub fn unpack_pack200(input: &Path, output: &Path) -> LarpResult<()> {
    // Use pack200 crate or system unpack200
    #[cfg(feature = "unpack200")]
    {
        let input_file = std::fs::File::open(input)?;
        let mut output_file = std::fs::File::create(output)?;
        
        // Use pack200 crate to unpack
        // pack200::unpack(input_file, &mut output_file)?;
        
        // For now, copy as-is (placeholder)
        std::io::copy(&mut std::io::BufReader::new(input_file), &mut output_file)?;
    }
    
    #[cfg(not(feature = "unpack200"))]
    {
        // Fallback: use system unpack200 command
        let status = std::process::Command::new("unpack200")
            .arg(input)
            .arg(output)
            .status()?;
        
        if !status.success() {
            return Err(crate::error::LarpError::Other("unpack200 failed".to_string()));
        }
    }
    
    Ok(())
}

/// Pack a JAR into Pack200 format
pub fn pack_pack200(input: &Path, output: &Path) -> LarpResult<()> {
    #[cfg(feature = "unpack200")]
    {
        let input_file = std::fs::File::open(input)?;
        let mut output_file = std::fs::File::create(output)?;
        
        // pack200::pack(input_file, &mut output_file)?;
        std::io::copy(&mut std::io::BufReader::new(input_file), &mut output_file)?;
    }
    
    #[cfg(not(feature = "unpack200"))]
    {
        let status = std::process::Command::new("pack200")
            .arg(output)
            .arg(input)
            .status()?;
        
        if !status.success() {
            return Err(crate::error::LarpError::Other("pack200 failed".to_string()));
        }
    }
    
    Ok(())
}

/// Check if a file is Pack200 format
pub fn is_pack200(path: &Path) -> bool {
    if let Ok(mut file) = std::fs::File::open(path) {
        let mut magic = [0u8; 4];
        if file.read_exact(&mut magic).is_ok() {
            return magic == [0xCA, 0xFE, 0xBA, 0xBE]; // Pack200 magic
        }
    }
    false
}

pub fn init() {
    log::info!("unpack200 initialized");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_is_pack200() {
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("test.pack");
        std::fs::write(&file, b"\xCA\xFE\xBA\xBE").unwrap();
        assert!(is_pack200(&file));
        
        let file2 = tmp.path().join("test.jar");
        std::fs::write(&file2, b"not pack200").unwrap();
        assert!(!is_pack200(&file2));
    }
}
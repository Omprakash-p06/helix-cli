use std::process::Command;
use std::path::{Path, PathBuf};
use std::fs;

#[derive(Debug)]
pub enum SnapshotError {
    PlatformNotSupported(String),
    CommandFailed(String),
    IoError(std::io::Error),
}

impl From<std::io::Error> for SnapshotError {
    fn from(err: std::io::Error) -> Self {
        SnapshotError::IoError(err)
    }
}

pub struct SnapshotManager {
    backup_dir: PathBuf,
    sources: Vec<PathBuf>,
}

impl SnapshotManager {
    pub fn new<P: AsRef<Path>>(backup_dir: P) -> Self {
        Self {
            backup_dir: backup_dir.as_ref().to_path_buf(),
            sources: vec![PathBuf::from("/etc")],
        }
    }

    #[cfg(test)]
    pub fn with_sources(backup_dir: PathBuf, sources: Vec<PathBuf>) -> Self {
        Self {
            backup_dir,
            sources,
        }
    }

    pub fn create_snapshot(&self) -> Result<String, SnapshotError> {
        #[cfg(target_os = "windows")]
        {
            let output = Command::new("vssadmin")
                .args(["create", "shadow", "/for=C:"])
                .output()?;

            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(SnapshotError::CommandFailed(String::from_utf8_lossy(&output.stderr).to_string()))
            }
        }

        #[cfg(target_os = "linux")]
        {
            if !self.backup_dir.exists() {
                fs::create_dir_all(&self.backup_dir)?;
            }

            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
            let snapshot_file = self.backup_dir.join(format!("snapshot_{}.tar.gz", timestamp));

            let mut cmd = Command::new("tar");
            cmd.arg("-czf").arg(&snapshot_file);
            
            for source in &self.sources {
                if source.exists() {
                    cmd.arg(source);
                }
            }

            let output = cmd.output()?;

            if output.status.success() {
                Ok(snapshot_file.to_string_lossy().to_string())
            } else {
                Err(SnapshotError::CommandFailed(String::from_utf8_lossy(&output.stderr).to_string()))
            }
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            Err(SnapshotError::PlatformNotSupported(std::env::consts::OS.to_string()))
        }
    }

    pub fn restore_snapshot(&self, snapshot_id: &str) -> Result<(), SnapshotError> {
        #[cfg(target_os = "windows")]
        {
            // Windows restoration via VSS is complex and usually requires external tools or specific APIs.
            // For now, we just log that it's requested.
            Err(SnapshotError::PlatformNotSupported("VSS restore not yet implemented in MVP".to_string()))
        }

        #[cfg(target_os = "linux")]
        {
            let snapshot_path = Path::new(snapshot_id);
            if !snapshot_path.exists() {
                return Err(SnapshotError::CommandFailed("Snapshot file does not exist".to_string()));
            }

            let output = Command::new("tar")
                .args([
                    "-xzf",
                    snapshot_path.to_str().unwrap(),
                    "-C",
                    "/",
                ])
                .output()?;

            if output.status.success() {
                Ok(())
            } else {
                Err(SnapshotError::CommandFailed(String::from_utf8_lossy(&output.stderr).to_string()))
            }
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            Err(SnapshotError::PlatformNotSupported(std::env::consts::OS.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;

    #[test]
    #[cfg(target_os = "linux")]
    fn test_linux_snapshot_creation() {
        let backup_dir = tempdir().unwrap();
        let source_dir = tempdir().unwrap();
        
        // Create a dummy file to backup
        let file_path = source_dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, world!").unwrap();

        let manager = SnapshotManager::with_sources(
            backup_dir.path().to_path_buf(),
            vec![source_dir.path().to_path_buf()]
        );
        
        let result = manager.create_snapshot();
        
        match result {
            Ok(path) => {
                assert!(path.contains("snapshot_"));
                assert!(path.ends_with(".tar.gz"));
                assert!(Path::new(&path).exists());
            },
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
}

use anyhow::{anyhow, Result};
use flate2::read::GzDecoder;
use semver::Version;
use serde::Deserialize;
use std::env;
use std::io::Cursor;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use tar::Archive;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const GITHUB_API_BASE: &str = "https://api.github.com";
const REPO_OWNER: &str = "aymenhamada";
const REPO_NAME: &str = "Commayte";

#[derive(Debug, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub body: String,
    pub assets: Vec<GitHubAsset>,
}

impl GitHubRelease {
    pub fn version(&self) -> &str {
        self.tag_name.trim_start_matches('v')
    }
}

#[derive(Debug, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
}

pub fn check_for_updates() -> Result<Option<GitHubRelease>> {
    println!("🔍 Checking for updates...");

    let client = reqwest::blocking::Client::new();
    let url = format!("{GITHUB_API_BASE}/repos/{REPO_OWNER}/{REPO_NAME}/releases/latest");

    let response = client
        .get(&url)
        .header("User-Agent", "Commayte-Update-Checker")
        .send()?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Failed to fetch release info: {}",
            response.status()
        ));
    }

    let release: GitHubRelease = response.json()?;
    let latest_version = release.tag_name.trim_start_matches('v');

    let current = Version::parse(CURRENT_VERSION)?;
    let latest = Version::parse(latest_version)?;

    if latest > current {
        println!("✨ New version available: v{latest_version}");
        println!("📝 Release notes: {}", release.name);

        // Check if the release has the expected tar.gz asset (like install.sh expects)
        let tar_asset_name = format!("commayte-{}.tar.gz", release.tag_name);
        let has_tar_asset = release.assets.iter().any(|a| a.name == tar_asset_name);

        if !has_tar_asset {
            println!("⚠️  Warning: Release doesn't contain the expected tar.gz archive");
            println!("💡 This might be a pre-release or development version");
        }

        Ok(Some(release))
    } else {
        println!("✅ You're running the latest version (v{CURRENT_VERSION})");
        Ok(None)
    }
}

pub fn perform_update(release: &GitHubRelease) -> Result<()> {
    println!("🚀 Starting update to v{}", release.version());

    let platform = get_platform_identifier()?;
    let os = std::env::consts::OS;

    // Download the new binary using the same logic as install.sh
    let new_binary = download_and_extract_binary(release, &platform)?;

    // Handle installation based on platform (same logic as install.sh)
    if os == "windows" {
        // Windows: Install to Program Files
        let install_dir = PathBuf::from("C:\\Program Files\\Commayte");
        std::fs::create_dir_all(&install_dir)?;

        let install_path = install_dir.join("commayte.exe");
        std::fs::write(&install_path, new_binary)?;

        println!("✅ Updated to: {install_path:?}");
        println!("📝 Note: Make sure 'C:\\Program Files\\Commayte' is in your PATH");
    } else {
        // Unix-like: Install to /usr/local/bin (requires sudo)
        let install_path = PathBuf::from("/usr/local/bin/commayte");

        match std::fs::write(&install_path, &new_binary) {
            Ok(_) => {
                #[cfg(unix)]
                {
                    let mut perms = std::fs::metadata(&install_path)?.permissions();
                    perms.set_mode(0o755);
                    std::fs::set_permissions(&install_path, perms)?;
                }
                println!("✅ Updated to: {install_path:?}");
            }
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                println!("🔐 Permission denied: attempting with sudo...");

                // Write to a temporary file first
                let temp_path = "/tmp/commayte-update.tmp";
                std::fs::write(temp_path, &new_binary)?;

                #[cfg(unix)]
                {
                    let mut perms = std::fs::metadata(temp_path)?.permissions();
                    perms.set_mode(0o755);
                    std::fs::set_permissions(temp_path, perms)?;
                }

                // Try to move it with sudo
                let status = Command::new("sudo")
                    .arg("mv")
                    .arg(temp_path)
                    .arg(&install_path)
                    .status()?;

                if status.success() {
                    println!("✅ Successfully installed to: {install_path:?}");
                } else {
                    return Err(anyhow!("❌ Failed to install binary with sudo."));
                }
            }
            Err(e) => {
                return Err(anyhow!("Failed to write binary: {}", e));
            }
        }
    }

    println!("🔄 Please restart the application to use the new version.");

    Ok(())
}

fn download_and_extract_binary(release: &GitHubRelease, platform: &str) -> Result<Vec<u8>> {
    // Find the tar.gz asset (same as install.sh)
    let tar_asset_name = format!("commayte-{}.tar.gz", release.tag_name);
    let tar_asset = release
        .assets
        .iter()
        .find(|a| a.name == tar_asset_name)
        .ok_or_else(|| anyhow!("No tar.gz archive found in release"))?;

    println!("📦 Downloading release archive: {}", tar_asset.name);

    // Download the tar.gz file
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&tar_asset.browser_download_url)
        .header("User-Agent", "Commayte-Updater")
        .send()?;

    if !response.status().is_success() {
        return Err(anyhow!("Failed to download archive: {}", response.status()));
    }

    let tar_data = response.bytes()?;

    // Extract the specific binary from the tar.gz (same logic as install.sh)
    let expected_path = format!("{platform}/{platform}");
    println!("🔍 Extracting binary from: {expected_path}");

    // Create a cursor to read the tar.gz data
    let cursor = Cursor::new(&tar_data);
    let gz = GzDecoder::new(cursor);
    let mut archive = Archive::new(gz);

    // Extract the specific binary
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;

        if path.to_string_lossy() == expected_path {
            let mut binary_data = Vec::new();
            std::io::copy(&mut entry, &mut binary_data)?;
            println!("✅ Successfully extracted binary from tar.gz");
            return Ok(binary_data);
        }
    }

    // If we can't find the binary in the tar.gz, fallback to direct download
    println!("⚠️  Binary not found in tar.gz, trying direct download...");

    let binary_asset = release
        .assets
        .iter()
        .find(|a| a.name == platform)
        .ok_or_else(|| anyhow!("No compatible binary found for platform: {}", platform))?;

    println!("📦 Downloading binary: {}", binary_asset.name);

    let binary_response = client
        .get(&binary_asset.browser_download_url)
        .header("User-Agent", "Commayte-Updater")
        .send()?;

    if !binary_response.status().is_success() {
        return Err(anyhow!(
            "Failed to download binary: {}",
            binary_response.status()
        ));
    }

    Ok(binary_response.bytes()?.to_vec())
}

fn get_platform_identifier() -> Result<String> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    match (os, arch) {
        ("linux", "x86_64") => Ok("commayte-linux-x86_64".to_string()),
        ("macos", "x86_64") => Ok("commayte-macos-x86_64".to_string()),
        ("macos", "aarch64") => Ok("commayte-macos-arm64".to_string()),
        ("windows", "x86_64") => Ok("commayte-windows-x86_64.exe".to_string()),
        _ => Err(anyhow!("Unsupported platform: {}-{}", os, arch)),
    }
}

pub fn show_update_info() {
    println!("📋 Update Information:");
    println!("   Current version: v{CURRENT_VERSION}");
    println!("   Repository: https://github.com/{REPO_OWNER}/{REPO_NAME}");
    println!("   Check for updates: commayte update");
    println!("   Auto-update: commayte update --auto");
}

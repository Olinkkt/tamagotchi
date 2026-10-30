//! Finding and installing a newer version of tama. The newest version wins: your own source folder
//! while you are ahead of GitHub, GitHub for everyone who downloaded the game from there.
use std::path::PathBuf;
use std::process::Command;

/// GitHub repo (`owner/name`) whose newest release is the update when the local source folder is not newer.
const REPO: &str = "Olinkkt/tamagotchi";
const ASSET: &str = "tama-linux-x86_64";

pub type Version = (u32, u32, u32);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Origin {
    Local,
    Github,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Found {
    pub version: Version,
    pub origin: Origin,
}

pub fn running() -> Version {
    parse_version(&format!("version = \"{}\"", env!("CARGO_PKG_VERSION"))).unwrap_or_default()
}

pub fn show(v: Version) -> String {
    format!("{}.{}.{}", v.0, v.1, v.2)
}

/// The version in the `version = "X.Y.Z"` line of a Cargo.toml.
pub fn parse_version(toml: &str) -> Option<Version> {
    let line = toml.lines().find(|l| l.trim_start().starts_with("version"))?;
    let text = line.split('"').nth(1)?;
    let mut parts = text.split('.').map(|p| p.parse::<u32>());
    Some((parts.next()?.ok()?, parts.next()?.ok()?, parts.next()?.ok()?))
}

/// The newest version that is newer than the running one. A tie goes to the local folder, which
/// needs no download.
pub fn pick(running: Version, local: Option<Version>, remote: Option<Version>) -> Option<Found> {
    let local = local.filter(|&v| v > running).map(|version| Found { version, origin: Origin::Local });
    let remote = remote.filter(|&v| v > running).map(|version| Found { version, origin: Origin::Github });
    match (local, remote) {
        (Some(l), Some(r)) if r.version > l.version => Some(r),
        (Some(l), _) => Some(l),
        (None, r) => r,
    }
}

fn source_dir() -> Option<PathBuf> {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    src.join("Cargo.toml").exists().then_some(src)
}

fn local_version() -> Option<Version> {
    parse_version(&std::fs::read_to_string(source_dir()?.join("Cargo.toml")).ok()?)
}

/// The newest release on GitHub, read from where `/releases/latest` redirects to (no API needed),
/// asked with curl so the game has no networking code. None when offline or there is no release.
fn remote_version() -> Option<Version> {
    let url = format!("https://github.com/{REPO}/releases/latest");
    let out = Command::new("curl").args(["-fsSL", "--max-time", "8", "-o", "/dev/null", "-w", "%{url_effective}", &url]).output().ok()?;
    let landed = String::from_utf8_lossy(&out.stdout);
    let tag = landed.rsplit_once("/tag/v")?.1;
    parse_version(&format!("version = \"{}\"", tag.trim()))
}

/// Is there a newer version anywhere? Slow (it uses the network): run it off the main thread.
pub fn check() -> Option<Found> {
    pick(running(), local_version(), remote_version())
}

/// `cargo` next to the installed `tama` (`~/.cargo/bin`), since a launcher may not have it on PATH.
fn cargo() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| Some(exe.parent()?.join("cargo")))
        .filter(|p| p.exists())
        .unwrap_or_else(|| PathBuf::from("cargo"))
}

/// Install the newest version: build the local folder if it is ahead, else download the GitHub
/// release. With nothing newer, rebuild the local folder if there is one. Returns an exit code.
pub fn install() -> i32 {
    match check() {
        Some(Found { origin: Origin::Github, version }) => {
            println!("Downloading tama {} from github.com/{REPO}", show(version));
            download()
        }
        Some(_) => build_local(),
        None if source_dir().is_some() => build_local(),
        None => {
            println!("tama {} is up to date.", show(running()));
            0
        }
    }
}

/// Rebuild from the source folder with cargo.
fn build_local() -> i32 {
    let Some(src) = source_dir() else { return 1 };
    println!("Rebuilding from {}", src.display());
    // --locked builds with the versions in Cargo.lock, which are known to work.
    match Command::new(cargo()).args(["install", "--force", "--locked", "--path"]).arg(src).status() {
        Ok(s) if s.success() => 0,
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => {
            eprintln!("Could not run cargo ({e}). Is Rust installed and on your PATH?");
            1
        }
    }
}

/// Replace this program with the newest release binary. It is checked before it is moved into place.
fn download() -> i32 {
    if !cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        eprintln!("There is no download for this system yet; build it with cargo install --git https://github.com/{REPO}");
        return 1;
    }
    let Ok(exe) = std::env::current_exe() else { return 1 };
    let new = exe.with_extension("new");
    let url = format!("https://github.com/{REPO}/releases/latest/download/{ASSET}");
    let fetched = Command::new("curl").args(["-fsSL", "--max-time", "120", "-o"]).arg(&new).arg(&url).status();
    let installed = fetched.is_ok_and(|s| s.success()) && make_runnable(&new) && std::fs::rename(&new, &exe).is_ok();
    if !installed {
        let _ = std::fs::remove_file(&new);
        eprintln!("Could not download the update. Check your connection and try again.");
        return 1;
    }
    0
}

/// Mark the downloaded file executable and make sure it starts.
fn make_runnable(path: &std::path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).is_ok()
        && Command::new(path).arg("--version").output().is_ok_and(|o| o.status.success())
}

/// Update in the background after the game has closed, then start the game again. The output goes
/// to `update.log` in the data folder; if the update fails the old version simply starts again.
#[cfg(unix)]
pub fn install_and_relaunch() {
    use std::os::unix::process::CommandExt;
    let Ok(exe) = std::env::current_exe() else { return };
    let dir = crate::save::dir();
    let _ = std::fs::create_dir_all(&dir);
    let log = std::fs::File::create(dir.join("update.log"));
    let (out, err) = match log.and_then(|f| Ok((f.try_clone()?, f))) {
        Ok((a, b)) => (std::process::Stdio::from(a), std::process::Stdio::from(b)),
        Err(_) => (std::process::Stdio::null(), std::process::Stdio::null()),
    };
    let _ = Command::new("sh")
        .args(["-c", "\"$0\" --update; exec \"$0\""])
        .arg(exe)
        .stdin(std::process::Stdio::null())
        .stdout(out)
        .stderr(err)
        .process_group(0)
        .spawn();
}

#[cfg(not(unix))]
pub fn install_and_relaunch() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_package_version() {
        assert_eq!(parse_version("[package]\nname = \"x\"\nversion = \"1.19.0\"\n"), Some((1, 19, 0)));
        assert_eq!(parse_version("nothing here"), None);
        assert_eq!(parse_version("version = \"1.x.0\""), None);
    }

    #[test]
    fn the_newest_version_wins_and_a_tie_goes_to_the_local_folder() {
        let run = (1, 18, 0);
        let found = |version, origin| Some(Found { version, origin });
        assert_eq!(pick(run, Some((1, 19, 0)), Some((1, 18, 0))), found((1, 19, 0), Origin::Local), "ahead of GitHub");
        assert_eq!(pick(run, Some((1, 18, 0)), Some((1, 19, 0))), found((1, 19, 0), Origin::Github), "a downloaded copy");
        assert_eq!(pick(run, Some((1, 19, 0)), Some((1, 19, 0))), found((1, 19, 0), Origin::Local));
        assert_eq!(pick(run, None, Some((1, 18, 1))), found((1, 18, 1), Origin::Github));
        assert_eq!(pick(run, Some((1, 18, 0)), None), None, "already up to date");
        assert_eq!(pick(run, None, None), None, "offline and no source folder");
    }
}

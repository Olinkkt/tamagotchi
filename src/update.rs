//! Finding and installing a newer version of tama. The newest version wins: your own source folder
//! while you are ahead of GitHub, GitHub for everyone who downloaded the game from there.
use std::path::PathBuf;
use std::process::Command;

/// GitHub repo (`owner/name`) that updates come from when the local source folder is not newer.
const REPO: &str = "Olinkkt/tamagotchi";
const BRANCH: &str = "master";

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

/// The version on GitHub, asked with curl so the game needs no networking code. None when offline.
fn remote_version() -> Option<Version> {
    let url = format!("https://raw.githubusercontent.com/{REPO}/{BRANCH}/Cargo.toml");
    let out = Command::new("curl").args(["-fsS", "--max-time", "8", &url]).output().ok()?;
    out.status.success().then(|| parse_version(&String::from_utf8_lossy(&out.stdout))).flatten()
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

/// Reinstall from the newest source. With nothing newer, rebuild from the local folder if there is
/// one, else from GitHub. Returns an exit code.
pub fn install() -> i32 {
    let origin = check().map(|f| f.origin).unwrap_or(if source_dir().is_some() { Origin::Local } else { Origin::Github });
    let mut cmd = Command::new(cargo());
    // --locked builds with the versions in Cargo.lock, which are known to work.
    cmd.args(["install", "--force", "--locked"]);
    match (origin, source_dir()) {
        (Origin::Local, Some(src)) => {
            println!("Rebuilding from {}", src.display());
            cmd.arg("--path").arg(src);
        }
        _ => {
            println!("Installing the newest version from github.com/{REPO}");
            cmd.args(["--git", &format!("https://github.com/{REPO}"), "--branch", BRANCH]);
        }
    }
    match cmd.status() {
        Ok(s) if s.success() => 0,
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => {
            eprintln!("Could not run cargo ({e}). Is Rust installed and on your PATH?");
            1
        }
    }
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

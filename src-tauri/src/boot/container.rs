use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime};

use super::PROD_IDENTIFIER;

pub const STAY_ENV: &str = "SYNK_CONTENEUR";
const RELAUNCH_MARKER: &str = "relance-hors-conteneur";
const LOOP_GUARD: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Launch {
    Outside,
    Relaunched,
    StayedInside(PathBuf),
    Skipped(&'static str),
    Unknown(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Decision {
    Run,
    Relaunch,
    StayInside,
}

static LAUNCH: OnceLock<Launch> = OnceLock::new();

pub fn decide(in_layer: bool, relaunched_recently: bool) -> Decision {
    match (in_layer, relaunched_recently) {
        (false, _) => Decision::Run,
        (true, false) => Decision::Relaunch,
        (true, true) => Decision::StayInside,
    }
}

pub fn layer_holding(packages: &Path, relative: &Path) -> Option<PathBuf> {
    std::fs::read_dir(packages)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| {
            entry
                .path()
                .join("LocalCache")
                .join("Roaming")
                .join(relative)
        })
        .find(|candidate| candidate.exists())
}

pub fn relaunched_recently(marker: &Path, now: SystemTime) -> bool {
    std::fs::metadata(marker)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|written| now.duration_since(written).ok())
        .is_some_and(|age| age < LOOP_GUARD)
}

pub fn probe(data_dir: &Path, packages: &Path) -> std::io::Result<Option<PathBuf>> {
    std::fs::create_dir_all(data_dir)?;
    let name = format!("sonde-calque-{}.tmp", std::process::id());
    let file = data_dir.join(&name);
    std::fs::write(&file, b"")?;
    let found = layer_holding(packages, &Path::new(PROD_IDENTIFIER).join(&name));
    std::fs::remove_file(&file)?;
    Ok(found.and_then(|path| path.parent().map(Path::to_path_buf)))
}

fn relaunch(data_dir: &Path) -> std::io::Result<()> {
    std::fs::write(data_dir.join(RELAUNCH_MARKER), b"")?;
    let exe = std::env::current_exe()?;
    let explorer = std::env::var_os("WINDIR")
        .map(|dir| PathBuf::from(dir).join("explorer.exe"))
        .unwrap_or_else(|| PathBuf::from("explorer.exe"));
    std::process::Command::new(explorer)
        .arg(exe)
        .spawn()
        .map(drop)
}

fn evaluate() -> Launch {
    if cfg!(debug_assertions) {
        return Launch::Skipped("build de dev");
    }
    if std::env::var(STAY_ENV).as_deref() == Ok("rester") {
        return Launch::Skipped("SYNK_CONTENEUR=rester");
    }
    let (Some(roaming), Some(local)) = (
        std::env::var_os("APPDATA"),
        std::env::var_os("LOCALAPPDATA"),
    ) else {
        return Launch::Unknown("APPDATA ou LOCALAPPDATA absent, conteneur non sonde".into());
    };
    let data_dir = PathBuf::from(roaming).join(PROD_IDENTIFIER);
    let packages = PathBuf::from(local).join("Packages");
    let layer = match probe(&data_dir, &packages) {
        Ok(layer) => layer,
        Err(e) => return Launch::Unknown(format!("sonde du conteneur impossible : {e}")),
    };
    let recent = relaunched_recently(&data_dir.join(RELAUNCH_MARKER), SystemTime::now());
    match (decide(layer.is_some(), recent), layer) {
        (Decision::Relaunch, _) => match relaunch(&data_dir) {
            Ok(()) => Launch::Relaunched,
            Err(e) => Launch::Unknown(format!("relance hors conteneur impossible : {e}")),
        },
        (Decision::StayInside, Some(layer)) => Launch::StayedInside(layer),
        (Decision::Run | Decision::StayInside, _) => Launch::Outside,
    }
}

pub fn relaunch_outside_if_needed() -> bool {
    let launch = evaluate();
    let relaunched = launch == Launch::Relaunched;
    LAUNCH.set(launch).ok();
    relaunched
}

pub fn log_launch() {
    match LAUNCH.get() {
        Some(Launch::Outside) => log::info!("[boot] lancement hors conteneur"),
        Some(Launch::Skipped(reason)) => {
            log::info!("[boot] detection du conteneur sautee : {reason}")
        }
        Some(Launch::StayedInside(layer)) => log::warn!(
            "[boot] Synk tourne dans le conteneur d'une application ({}) malgre une relance de moins de 60 s : ses ecritures restent dans ce calque, le lancer par son raccourci",
            layer.display()
        ),
        Some(Launch::Unknown(reason)) => log::warn!("[boot] {reason}"),
        Some(Launch::Relaunched) | None => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_layer_without_a_recent_relaunch_relaunches() {
        assert_eq!(decide(false, false), Decision::Run);
        assert_eq!(decide(false, true), Decision::Run);
        assert_eq!(decide(true, false), Decision::Relaunch);
        assert_eq!(decide(true, true), Decision::StayInside);
    }

    #[test]
    fn the_layer_is_the_package_cache_that_holds_the_probe() {
        let packages = tempfile::tempdir().unwrap();
        let relative = Path::new(PROD_IDENTIFIER).join("sonde-calque-1.tmp");
        std::fs::create_dir_all(packages.path().join("Autre_x1").join("LocalCache")).unwrap();
        assert_eq!(layer_holding(packages.path(), &relative), None);
        let held = packages
            .path()
            .join("Claude_abc")
            .join("LocalCache")
            .join("Roaming")
            .join(&relative);
        std::fs::create_dir_all(held.parent().unwrap()).unwrap();
        std::fs::write(&held, b"").unwrap();
        assert_eq!(layer_holding(packages.path(), &relative), Some(held));
    }

    #[test]
    fn a_missing_package_folder_is_no_layer() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            layer_holding(&dir.path().join("absent"), Path::new("x")),
            None
        );
    }

    #[test]
    fn the_loop_guard_lasts_sixty_seconds() {
        let dir = tempfile::tempdir().unwrap();
        let marker = dir.path().join(RELAUNCH_MARKER);
        let now = SystemTime::now();
        assert!(!relaunched_recently(&marker, now));
        std::fs::write(&marker, b"").unwrap();
        assert!(relaunched_recently(&marker, now + Duration::from_secs(5)));
        assert!(!relaunched_recently(&marker, now + Duration::from_secs(61)));
    }

    #[test]
    fn the_probe_leaves_nothing_behind_outside_a_layer() {
        let data = tempfile::tempdir().unwrap();
        let packages = tempfile::tempdir().unwrap();
        assert_eq!(probe(data.path(), packages.path()).unwrap(), None);
        assert_eq!(std::fs::read_dir(data.path()).unwrap().count(), 0);
    }
}

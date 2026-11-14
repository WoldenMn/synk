use crate::support::temp_dirs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

const FORBIDDEN: [&str; 6] = [
    "tempfile::tempdir()",
    "temp_dir()",
    ".tempfile()",
    ".tempdir()",
    "NamedTempFile::new()",
    "TempDir::new()",
];

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("dossier") {
        let path = entry.expect("entree").path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

#[test]
fn no_integration_test_writes_into_the_system_temp_dir() {
    let tests = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let allowed = [
        tests.join("integration").join("temp_dirs.rs"),
        tests
            .join("integration")
            .join("support")
            .join("temp_dirs.rs"),
    ];
    let mut files = Vec::new();
    rust_files(&tests, &mut files);
    assert!(files.len() > 50, "{} fichiers lus", files.len());
    let mut offenders = Vec::new();
    for path in files.iter().filter(|p| !allowed.contains(p)) {
        let text = std::fs::read_to_string(path).expect("lecture");
        for pattern in FORBIDDEN {
            if text.contains(pattern) {
                offenders.push(format!("{} : {pattern}", path.display()));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "passer par support::temp_dirs : {offenders:?}"
    );
}

#[test]
fn a_test_directory_lives_under_the_cargo_target_tmpdir() {
    let dir = temp_dirs::tempdir().unwrap();
    assert!(dir.path().starts_with(env!("CARGO_TARGET_TMPDIR")));
    assert!(dir.path().starts_with(temp_dirs::root()));
    assert!(dir.path().is_dir());
}

#[test]
fn the_sweep_removes_only_what_is_older_than_the_cutoff() {
    let dir = temp_dirs::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("ancien")).unwrap();
    std::fs::write(dir.path().join("ancien").join("synk.db"), b"x").unwrap();
    std::fs::write(dir.path().join("export.json"), b"{}").unwrap();

    let before = SystemTime::now() - Duration::from_secs(3600);
    assert_eq!(temp_dirs::sweep(dir.path(), before).unwrap(), 0);
    assert!(dir.path().join("ancien").join("synk.db").is_file());
    assert!(dir.path().join("export.json").is_file());

    let after = SystemTime::now() + Duration::from_secs(3600);
    assert_eq!(temp_dirs::sweep(dir.path(), after).unwrap(), 2);
    assert!(std::fs::read_dir(dir.path()).unwrap().next().is_none());
}

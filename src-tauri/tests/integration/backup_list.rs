use std::time::{Duration, SystemTime};
use synk_lib::services::backup_files::list_backups_inner;

fn write_backup(path: &std::path::Path, age_secs: u64) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, b"contenu").unwrap();
    let file = std::fs::File::options().write(true).open(path).unwrap();
    file.set_modified(SystemTime::now() - Duration::from_secs(age_secs))
        .unwrap();
}

#[test]
fn lists_every_kind_newest_first() {
    let dir = tempfile::tempdir().unwrap();
    let backups = dir.path().join("backups");
    write_backup(&backups.join("auto").join("a.db"), 300);
    write_backup(&backups.join("daily").join("b.db"), 100);
    write_backup(&backups.join("c.db"), 200);
    let list = list_backups_inner(dir.path()).unwrap();
    let kinds: Vec<&str> = list.iter().map(|b| b.kind).collect();
    assert_eq!(kinds, ["quotidienne", "manuelle", "auto"]);
    assert_eq!(list[0].size_bytes, 7);
    assert!(
        list[0].modified.ends_with('Z'),
        "obtenu : {}",
        list[0].modified
    );
}

#[test]
fn no_backup_folder_gives_an_empty_list() {
    let dir = tempfile::tempdir().unwrap();
    assert!(list_backups_inner(dir.path()).unwrap().is_empty());
}

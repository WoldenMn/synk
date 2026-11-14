use std::io;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime};

pub const STALE_AFTER: Duration = Duration::from_secs(3600);

pub fn root() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("essais");
        std::fs::create_dir_all(&root)
            .unwrap_or_else(|e| panic!("dossier des essais {} : {e}", root.display()));
        let cutoff = SystemTime::now()
            .checked_sub(STALE_AFTER)
            .unwrap_or(SystemTime::UNIX_EPOCH);
        if let Err(e) = sweep(&root, cutoff) {
            eprintln!("essais d'avant laisses dans {} : {e}", root.display());
        }
        root
    })
}

pub fn tempdir() -> io::Result<tempfile::TempDir> {
    tempfile::Builder::new().prefix("essai-").tempdir_in(root())
}

pub fn sweep(dir: &Path, cutoff: SystemTime) -> io::Result<usize> {
    let mut removed = 0;
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        if entry.metadata()?.modified()? >= cutoff {
            continue;
        }
        let path = entry.path();
        let gone = if entry.file_type()?.is_dir() {
            std::fs::remove_dir_all(&path)
        } else {
            std::fs::remove_file(&path)
        };
        match gone {
            Ok(()) => removed += 1,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => eprintln!("essai laisse {} : {e}", path.display()),
        }
    }
    Ok(removed)
}

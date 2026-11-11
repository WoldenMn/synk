mod backup;
mod backup_files;
mod csv;
mod json;
mod pdf;
mod restore;
mod shared;

// IMPORTANT : glob `pub use module::*` (pas de re-exports explicites).
// `tauri::generate_handler!` (utilise dans lib.rs) genere un identifiant cache
// `__cmd__<name>` au cote de chaque fonction `#[tauri::command]`. Un re-export
// explicite (`pub use csv::export_csv;`) ne re-exporte que le nom public et
// laisse `__cmd__export_csv` introuvable depuis `commands::export::*`,
// ce qui casse `generate_handler!`. Le glob re-exporte les deux. Ne pas
// "nettoyer" en re-exports nommes : ca casserait le build.
pub use backup::*;
pub use backup_files::*;
pub use csv::*;
pub use json::*;
pub use pdf::*;
pub use restore::*;

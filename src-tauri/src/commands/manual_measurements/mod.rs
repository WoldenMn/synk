mod goals;
mod measurements;
mod stats;
mod types;

// IMPORTANT : glob `pub use module::*` (pas de re-exports nommes).
// `tauri::generate_handler!` (utilise dans lib.rs) genere un identifiant cache
// `__cmd__<name>` au cote de chaque fonction `#[tauri::command]`. Un re-export
// explicite ne re-exporte que le nom public et laisse `__cmd__<name>` introuvable.
// Le glob re-exporte les deux. Ne pas "nettoyer" en re-exports nommes.
pub use goals::*;
pub use measurements::*;
pub use stats::*;
pub use types::*;

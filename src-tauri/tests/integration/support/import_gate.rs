use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

static GATE: RwLock<()> = RwLock::const_new(());

pub async fn importing() -> RwLockReadGuard<'static, ()> {
    GATE.read().await
}

pub async fn no_import_running() -> RwLockWriteGuard<'static, ()> {
    GATE.write().await
}

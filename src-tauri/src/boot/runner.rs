use crate::error::AppError;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::RwLock;

/// Snapshot du boot, lu de maniere atomique par le frontend pour rattraper le cas
/// ou l'event `app-ready` ou `app-error` aurait ete emis avant que les listeners
/// soient attaches (race condition fire-and-forget des events Tauri).
#[derive(Clone, Debug, serde::Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum BootSnapshot {
    Loading,
    Ready {
        integrity_warning: Option<super::IntegrityWarning>,
    },
    Failed {
        error: super::BootError,
    },
}

pub struct AppBootState(pub Arc<RwLock<BootSnapshot>>);

fn emit_logged<S: serde::Serialize + Clone>(handle: &AppHandle, event: &str, payload: S) {
    if let Err(e) = handle.emit(event, payload) {
        log::warn!("[boot] emission de {event} impossible : {e}");
    }
}

/// Lance `boot::run` en tache detachee et publie son issue dans le snapshot,
/// puis par evenement. Appele au demarrage, et de nouveau par `retry_boot`
/// apres un echec : l'etat Tauri (pool, coffre, secrets invalides) n'est gere
/// que sur un boot reussi, il n'existe donc jamais deux fois.
pub fn spawn_boot(handle: AppHandle, snapshot: Arc<RwLock<BootSnapshot>>) {
    tauri::async_runtime::spawn(async move {
        log::info!("[boot] async task started");
        let started_at = std::time::Instant::now();
        match super::run(handle.clone()).await {
            Ok(output) => {
                log::info!(
                    "[boot] boot::run completed OK in {}ms",
                    started_at.elapsed().as_millis()
                );
                handle.manage(output.pool.clone());
                handle.manage(output.vault.clone());
                handle.manage(output.invalidated_secrets);
                let warning = output.integrity_warning.clone();
                // Snapshot AVANT emit : garantit l'ordre causal
                // (get_boot_state pourra rattraper si event manque).
                *snapshot.write().await = BootSnapshot::Ready {
                    integrity_warning: warning.clone(),
                };
                log::info!("[boot] snapshot=Ready, emitting app-ready (defense in depth: 3 emits)");

                // Defense in depth : re-emit app-ready 3x (espace 200ms)
                // pour couvrir le cas ou la WebView se connecte tard.
                // Les listeners JS sont idempotents (matches!(prev, "loading")).
                for i in 0..3 {
                    emit_logged(&handle, "app-ready", ());
                    if let Some(w) = warning.clone() {
                        emit_logged(&handle, "app-integrity-warning", w);
                    }
                    log::info!("[boot] app-ready emit #{} sent", i + 1);
                    if i < 2 {
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                    }
                }

                super::spawn_post_ready_seed_then_backup(handle.clone(), output.pool.clone());
                super::spawn_post_ready_auto_purge(output.pool);
            }
            Err(e) => {
                log::error!(
                    "[boot] boot::run FAILED in {}ms: {:?}",
                    started_at.elapsed().as_millis(),
                    e
                );
                *snapshot.write().await = BootSnapshot::Failed { error: e.clone() };
                for i in 0..3 {
                    emit_logged(&handle, "app-error", &e);
                    log::info!("[boot] app-error emit #{} sent", i + 1);
                    if i < 2 {
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                    }
                }
            }
        }
    });
}

#[tauri::command]
pub(crate) async fn get_boot_state(
    state: tauri::State<'_, AppBootState>,
) -> Result<BootSnapshot, AppError> {
    Ok(state.0.read().await.clone())
}

pub(crate) async fn begin_retry(snapshot: &RwLock<BootSnapshot>) -> Result<(), AppError> {
    let mut current = snapshot.write().await;
    if !matches!(*current, BootSnapshot::Failed { .. }) {
        return Err(AppError::Internal(
            "Le démarrage n'est pas en échec, rien à relancer".into(),
        ));
    }
    *current = BootSnapshot::Loading;
    Ok(())
}

/// Relance le boot apres un echec (bouton Reessayer, ou suite a une
/// restauration). Refuse si le boot est en cours ou deja reussi : relancer un
/// boot reussi ouvrirait un second pool sur la meme base.
#[tauri::command]
pub async fn retry_boot(
    app: AppHandle,
    state: tauri::State<'_, AppBootState>,
) -> Result<(), AppError> {
    let snapshot = state.0.clone();
    begin_retry(&snapshot).await?;
    log::info!("[boot] retry_boot : nouvelle tentative de demarrage");
    spawn_boot(app, snapshot);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::boot::BootError;

    fn failed() -> BootSnapshot {
        BootSnapshot::Failed {
            error: BootError::BackupFailed {
                message: "disque plein".into(),
            },
        }
    }

    #[tokio::test]
    async fn a_failed_boot_can_be_retried_and_goes_back_to_loading() {
        let snapshot = RwLock::new(failed());

        begin_retry(&snapshot).await.unwrap();

        assert!(matches!(*snapshot.read().await, BootSnapshot::Loading));
    }

    #[tokio::test]
    async fn a_boot_in_progress_or_done_is_never_relaunched() {
        for state in [
            BootSnapshot::Loading,
            BootSnapshot::Ready {
                integrity_warning: None,
            },
        ] {
            let snapshot = RwLock::new(state);

            let err = begin_retry(&snapshot).await.unwrap_err();

            assert_eq!(err.kind(), "Internal");
            assert!(
                !matches!(*snapshot.read().await, BootSnapshot::Failed { .. }),
                "l'etat ne change pas"
            );
        }
    }

    #[test]
    fn the_snapshot_is_tagged_by_status_for_the_front() {
        let loading = serde_json::to_value(BootSnapshot::Loading).unwrap();
        let ready = serde_json::to_value(BootSnapshot::Ready {
            integrity_warning: None,
        })
        .unwrap();
        let failed = serde_json::to_value(failed()).unwrap();

        assert_eq!(loading["status"], "loading");
        assert_eq!(ready["status"], "ready");
        assert_eq!(failed["status"], "failed");
        assert_eq!(failed["error"]["kind"], "BackupFailed");
    }
}

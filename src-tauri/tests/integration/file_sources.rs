use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use synk_lib::sources::import::pipeline::ImportProgress;
use synk_lib::sources::{all_sources, ImportContext, SourceKind};

use crate::common::{memory_pool, test_vault};
use crate::support::import_gate;

#[tokio::test]
async fn every_file_source_carries_its_own_reader_and_refuses_to_sync() {
    let _gate = import_gate::importing().await;
    let pool = memory_pool().await;
    let vault = test_vault();
    let progress = |_: ImportProgress| {};
    for source in all_sources() {
        let ctx = || ImportContext {
            pool: &pool,
            path: "C:/dossier/introuvable.bin",
            cancel: Arc::new(AtomicBool::new(false)),
            on_progress: &progress,
        };
        match source.kind() {
            SourceKind::File(_) => {
                let future = source
                    .import(ctx())
                    .unwrap_or_else(|| panic!("{} sans lecteur", source.id().as_str()));
                let err = future.await.unwrap_err();
                assert_eq!(err.kind(), "Validation", "{}", source.id().as_str());

                let sync = source
                    .sync(&synk_lib::sources::SyncContext {
                        pool: &pool,
                        vault: &vault,
                        on_progress: &|_, _| {},
                    })
                    .await
                    .unwrap_err();
                assert_eq!(sync.kind(), "Validation");
                assert!(sync.to_string().contains("fichier"), "{sync}");
            }
            SourceKind::OAuth(_) => assert!(source.import(ctx()).is_none()),
        }
    }
}

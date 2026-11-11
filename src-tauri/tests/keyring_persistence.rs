#![cfg(windows)]

use keyring_core::{Entry, Error};

fn windows_store() {
    keyring_core::set_default_store(
        windows_native_keyring_store::Store::new().expect("magasin Windows"),
    );
}

const TEST_USER: &str = "persistence_probe";
const TEST_SECRET: &str = "valeur-de-sonde-non-sensible";

struct CredentialCleanup {
    service: String,
}

impl Drop for CredentialCleanup {
    fn drop(&mut self) {
        windows_store();
        let result = Entry::new(&self.service, TEST_USER).and_then(|e| e.delete_credential());
        match result {
            Ok(()) | Err(Error::NoEntry) => {}
            Err(e) => eprintln!(
                "nettoyage impossible de l'entree de test {}/{TEST_USER} : {e}",
                self.service
            ),
        }
    }
}

#[test]
fn entry_written_by_one_instance_is_read_by_a_distinct_instance() {
    windows_store();
    let service = format!("synk-test-persistence-{}", std::process::id());
    let cleanup = CredentialCleanup {
        service: service.clone(),
    };

    let writer = Entry::new(&service, TEST_USER).expect("creation de l'entree d'ecriture");
    writer
        .set_password(TEST_SECRET)
        .expect("ecriture dans le trousseau");

    let reader = Entry::new(&service, TEST_USER).expect("creation de l'entree de lecture");
    let read_back = reader.get_password().expect(
        "une seconde instance Entry doit relire l'entree : le store utilise ne persiste rien",
    );
    assert_eq!(read_back, TEST_SECRET);

    drop(cleanup);
    let after_cleanup = Entry::new(&service, TEST_USER)
        .expect("creation de l'entree de controle")
        .get_password();
    assert!(
        matches!(after_cleanup, Err(Error::NoEntry)),
        "l'entree de test aurait du etre supprimee, obtenu : {after_cleanup:?}"
    );
}

#[test]
fn stored_credential_keeps_the_keyring_3_target_and_persistence() {
    windows_store();
    let service = format!("synk-test-target-{}", std::process::id());
    let cleanup = CredentialCleanup {
        service: service.clone(),
    };
    let entry = Entry::new(&service, TEST_USER).expect("entree");
    entry.set_password(TEST_SECRET).expect("ecriture");
    let attributes = entry.get_attributes().expect("attributs");
    assert_eq!(
        attributes.get("target_name").map(String::as_str),
        Some(format!("{TEST_USER}.{service}").as_str())
    );
    assert_eq!(
        attributes.get("persistence").map(String::as_str),
        Some("Enterprise")
    );
    drop(cleanup);
}

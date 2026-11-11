use crate::support::activity_files::{
    sample_fit, sample_gpx, sample_libreview_csv, sample_scale_csv, sample_tcx, write,
};

#[test]
#[ignore = "ecrit des fichiers synthetiques pour la verification a l'ecran : SYNK_SAMPLE_FILES=<dossier>"]
fn write_sample_files_for_the_screen_check() {
    let Ok(target) = std::env::var("SYNK_SAMPLE_FILES") else {
        return;
    };
    let dir = std::path::PathBuf::from(target);
    std::fs::create_dir_all(&dir).unwrap();
    write(&dir, "sortie-synthetique.fit", &sample_fit());
    write(&dir, "sortie-synthetique.gpx", sample_gpx().as_bytes());
    write(&dir, "sortie-synthetique.tcx", sample_tcx().as_bytes());
    write(
        &dir,
        "balance-synthetique.csv",
        sample_scale_csv().as_bytes(),
    );
    write(
        &dir,
        "libreview-synthetique.csv",
        sample_libreview_csv().as_bytes(),
    );
}

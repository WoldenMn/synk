use synk_lib::services::insights::run_generation;

#[tokio::test]
async fn insert_insight_is_idempotent_per_day() {
    let pool = crate::common::memory_pool().await;
    synk_lib::services::insights::detectors::insert_insight(
        &pool,
        "trend",
        "info",
        Some("weight"),
        "Poids en baisse",
        "msg",
        Some(70.0),
        Some(-1.0),
    )
    .await
    .unwrap();
    synk_lib::services::insights::detectors::insert_insight(
        &pool,
        "trend",
        "info",
        Some("weight"),
        "Poids en baisse",
        "msg",
        Some(70.0),
        Some(-1.0),
    )
    .await
    .unwrap();
    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM insights WHERE kind='trend' AND metric='weight' AND date(generated_at)=date('now')"
    ).fetch_one(&pool).await.unwrap();
    assert_eq!(n, 1);
}

fn recent_iso(days_ago: i64) -> String {
    let d = chrono::Utc::now() - chrono::Duration::days(days_ago);
    d.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn recent_date_only(days_ago: i64) -> String {
    let d = chrono::Utc::now() - chrono::Duration::days(days_ago);
    d.format("%Y-%m-%d").to_string()
}

#[tokio::test]
async fn detects_insufficient_sleep() {
    let pool = crate::common::memory_pool().await;

    // Le detecteur lit les 3 dernieres nuits (<= 7 jours). On insere 3 nuits a 5h (18000s < 21600s).
    for i in 0..3 {
        sqlx::query(
            "INSERT INTO sleep_records (date, source, duration_total) VALUES (?1, 'withings', ?2)",
        )
        .bind(recent_date_only(i))
        .bind(18000_i64)
        .execute(&pool)
        .await
        .unwrap();
    }

    run_generation(&pool).await;

    let row: (String, String, Option<String>) = sqlx::query_as(
        "SELECT kind, severity, metric FROM insights WHERE metric = 'sleep' LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.0, "anomaly");
    assert_eq!(row.1, "warning");
    assert_eq!(row.2, Some("sleep".to_string()));
}

#[tokio::test]
async fn detects_hypertension() {
    let pool = crate::common::memory_pool().await;

    // Le detecteur : derniere mesure (ORDER BY measured_at DESC LIMIT 1) avec sys >= 140 OU dia >= 90.
    // On insere d'abord des mesures normales (hier-30j) puis une recente elevee.
    for i in 5..15 {
        sqlx::query(
            "INSERT INTO blood_pressure (measured_at, source, systolic, diastolic) VALUES (?1, 'manual', 120, 75)",
        )
        .bind(recent_iso(i))
        .execute(&pool)
        .await
        .unwrap();
    }
    // Derniere mesure = aujourd'hui, franchement hypertensive.
    sqlx::query(
        "INSERT INTO blood_pressure (measured_at, source, systolic, diastolic) VALUES (?1, 'manual', 160, 100)",
    )
    .bind(recent_iso(0))
    .execute(&pool)
    .await
    .unwrap();

    run_generation(&pool).await;

    let row: (String, String, Option<String>) = sqlx::query_as(
        "SELECT kind, severity, metric FROM insights WHERE metric = 'blood_pressure' LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.0, "anomaly");
    assert_eq!(row.1, "danger");
    assert_eq!(row.2, Some("blood_pressure".to_string()));
}

#[tokio::test]
async fn detects_heart_rate_anomaly() {
    let pool = crate::common::memory_pool().await;

    // Le detecteur calcule un Z-score sur les 30 derniers jours, avec min 10 mesures.
    // La "derniere valeur" est celle en fin de Vec (ordre d'insertion SQLite sans ORDER BY).
    // On insere 14 mesures a 60 bpm (tres stable) puis une mesure a 110 bpm en dernier.
    // heart_records a UNIQUE(timestamp, source) — timestamps distincts suffisent.
    for i in 1..15 {
        sqlx::query(
            "INSERT INTO heart_records (timestamp, source, heart_rate) VALUES (?1, 'manual', 60)",
        )
        .bind(recent_iso(i))
        .execute(&pool)
        .await
        .unwrap();
    }
    // Mesure anomale inseree en dernier (row_id max => derniere dans le Vec).
    sqlx::query(
        "INSERT INTO heart_records (timestamp, source, heart_rate) VALUES (?1, 'manual', 110)",
    )
    .bind(recent_iso(0))
    .execute(&pool)
    .await
    .unwrap();

    run_generation(&pool).await;

    let row: (String, String, Option<String>) = sqlx::query_as(
        "SELECT kind, severity, metric FROM insights WHERE metric = 'heart_rate' LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.0, "anomaly");
    assert_eq!(row.1, "warning");
    assert_eq!(row.2, Some("heart_rate".to_string()));
}

#[tokio::test]
async fn detects_activity_delta() {
    let pool = crate::common::memory_pool().await;

    // Le detecteur compare la moyenne des 7 derniers jours a celle des 7 jours precedents.
    // >= 15% de variation declenche l'insight. On cree une chute brutale (baseline eleve, puis bas).
    // activity a UNIQUE(date, source) — dates distinctes suffisent.
    // Semaine precedente (j-14 a j-8) : 12000 pas/jour
    for i in 8..15 {
        sqlx::query("INSERT INTO activity (date, source, steps) VALUES (?1, 'manual', 12000)")
            .bind(recent_date_only(i))
            .execute(&pool)
            .await
            .unwrap();
    }
    // Semaine recente (j-6 a j) : 3000 pas/jour -> -75%
    for i in 0..7 {
        sqlx::query("INSERT INTO activity (date, source, steps) VALUES (?1, 'manual', 3000)")
            .bind(recent_date_only(i))
            .execute(&pool)
            .await
            .unwrap();
    }

    run_generation(&pool).await;

    let row: (String, String, Option<String>) = sqlx::query_as(
        "SELECT kind, severity, metric FROM insights WHERE metric = 'steps' LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.0, "trend");
    assert_eq!(row.1, "warning");
    assert_eq!(row.2, Some("steps".to_string()));
}

// ── H3 : cas bords des détecteurs ─────────────────────────────────────────────

// ── Hypertension ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn hypertension_below_threshold_returns_0() {
    let pool = crate::common::memory_pool().await;
    // Derniere mesure juste sous le seuil : sys=139, dia=89 (tous les deux < 140 et < 90)
    for i in 0..5 {
        sqlx::query(
            "INSERT INTO blood_pressure (measured_at, source, systolic, diastolic) VALUES (?1, 'manual', 120, 75)",
        )
        .bind(recent_iso(i + 1))
        .execute(&pool)
        .await
        .unwrap();
    }
    sqlx::query(
        "INSERT INTO blood_pressure (measured_at, source, systolic, diastolic) VALUES (?1, 'manual', 139, 89)",
    )
    .bind(recent_iso(0))
    .execute(&pool)
    .await
    .unwrap();

    let n = synk_lib::services::insights::detectors::hypertension::detect(&pool)
        .await
        .unwrap();
    assert_eq!(
        n, 0,
        "sys=139/dia=89 est juste sous les seuils 140/90 → pas d'insight"
    );
}

#[tokio::test]
async fn hypertension_empty_table_returns_0() {
    let pool = crate::common::memory_pool().await;
    // Table blood_pressure vide : aucune donnée.
    let n = synk_lib::services::insights::detectors::hypertension::detect(&pool)
        .await
        .unwrap();
    assert_eq!(n, 0, "table vide → pas de panique, retourne 0");
}

// ── Heart rate anomaly ────────────────────────────────────────────────────────

#[tokio::test]
async fn heart_rate_anomaly_below_threshold_returns_0() {
    let pool = crate::common::memory_pool().await;
    // 14 mesures à 60 bpm + dernière à 70 bpm : z-score faible, pas d'anomalie (seuil z > 2.0).
    // std ≈ 2.9, z = (70-61.4)/2.9 ≈ 2.97 — pour rester sous le seuil : derniere = 65.
    // Avec 14 × 60 + 1 × 65 : mean ≈ 60.33, std ≈ 1.29, z ≈ 3.6 – trop proche.
    // Utiliser 14 × 60 + 1 × 63 : mean ≈ 60.2, std ≈ 0.75, z ≈ 3.7 – même problème.
    // Approche robuste : 50 × 60 + 1 × 70. mean ≈ 60.196, std ≈ 1.38, z ≈ 7.1 – trop.
    // Raisonner autrement : z ≤ 2.0 nécessite last ≤ mean + 2*std.
    // 10 × 60 + 1 × 61 → mean=60.09, var=0.082, std=0.287 → z=(61-60.09)/0.287≈3.17 → encore > 2.
    // Avec spread plus large : 5 × 55 + 5 × 75 + 1 × 76 → mean=65.54, std=9.07, z=(76-65.54)/9.07≈1.15 ≤ 2 ✓
    // Timestamps distincts : on utilise l'index de boucle pour éviter UNIQUE(timestamp, source).
    let hr_values: [i64; 10] = [55, 55, 55, 55, 55, 75, 75, 75, 75, 75];
    for (idx, &v) in hr_values.iter().enumerate() {
        sqlx::query(
            "INSERT INTO heart_records (timestamp, source, heart_rate) VALUES (?1, 'manual', ?2)",
        )
        .bind(recent_iso(idx as i64 + 2)) // décalage +2 pour laisser la place à la dernière mesure
        .bind(v)
        .execute(&pool)
        .await
        .unwrap();
    }
    // Dernière mesure : 76 bpm — z ≈ 1.15, sous le seuil 2.0.
    sqlx::query(
        "INSERT INTO heart_records (timestamp, source, heart_rate) VALUES (?1, 'manual', 76)",
    )
    .bind(recent_iso(1))
    .execute(&pool)
    .await
    .unwrap();

    let n = synk_lib::services::insights::detectors::heart_rate_anomaly::detect(&pool)
        .await
        .unwrap();
    assert_eq!(n, 0, "z-score ≈ 1.15 est sous le seuil 2.0 → pas d'insight");
}

#[tokio::test]
async fn heart_rate_anomaly_insufficient_data_returns_0() {
    let pool = crate::common::memory_pool().await;
    // Seulement 5 mesures (< 10 requises) → detect retourne 0 sans paniquer.
    for i in 0..5 {
        sqlx::query(
            "INSERT INTO heart_records (timestamp, source, heart_rate) VALUES (?1, 'manual', 110)",
        )
        .bind(recent_iso(i))
        .execute(&pool)
        .await
        .unwrap();
    }
    let n = synk_lib::services::insights::detectors::heart_rate_anomaly::detect(&pool)
        .await
        .unwrap();
    assert_eq!(n, 0, "< 10 mesures → seuil non atteint, retourne 0");
}

// ── Activity delta ────────────────────────────────────────────────────────────

#[tokio::test]
async fn activity_delta_below_threshold_returns_0() {
    let pool = crate::common::memory_pool().await;
    // Variation de seulement 10% (sous les 15% requis).
    // Semaine précédente : 10000 pas/jour
    for i in 8..15 {
        sqlx::query("INSERT INTO activity (date, source, steps) VALUES (?1, 'manual', 10000)")
            .bind(recent_date_only(i))
            .execute(&pool)
            .await
            .unwrap();
    }
    // Semaine récente : 10900 pas/jour → +9%, sous les 15%
    for i in 0..7 {
        sqlx::query("INSERT INTO activity (date, source, steps) VALUES (?1, 'manual', 10900)")
            .bind(recent_date_only(i))
            .execute(&pool)
            .await
            .unwrap();
    }
    let n = synk_lib::services::insights::detectors::activity_delta::detect(&pool)
        .await
        .unwrap();
    assert_eq!(n, 0, "variation +9% est sous le seuil 15% → pas d'insight");
}

#[tokio::test]
async fn activity_delta_empty_table_returns_0() {
    let pool = crate::common::memory_pool().await;
    // Table activity vide : COALESCE retourne 0.0 → condition `prev > 100.0` non satisfaite.
    let n = synk_lib::services::insights::detectors::activity_delta::detect(&pool)
        .await
        .unwrap();
    assert_eq!(n, 0, "table vide → pas de panique, retourne 0");
}

// ── Sleep insufficient ────────────────────────────────────────────────────────

#[tokio::test]
async fn sleep_insufficient_above_threshold_returns_0() {
    let pool = crate::common::memory_pool().await;
    // 3 nuits à 8h (28800s >= 21600s) → condition `r.1 < 21600` non satisfaite pour toutes.
    for i in 0..3 {
        sqlx::query(
            "INSERT INTO sleep_records (date, source, duration_total) VALUES (?1, 'withings', ?2)",
        )
        .bind(recent_date_only(i))
        .bind(28800_i64) // 8h en secondes
        .execute(&pool)
        .await
        .unwrap();
    }
    let n = synk_lib::services::insights::detectors::sleep_insufficient::detect(&pool)
        .await
        .unwrap();
    assert_eq!(n, 0, "8h de sommeil ≥ seuil 21600s → pas d'insight");
}

#[tokio::test]
async fn sleep_insufficient_empty_table_returns_0() {
    let pool = crate::common::memory_pool().await;
    // Aucune donnée → rows.len() != 3, pas d'insight.
    let n = synk_lib::services::insights::detectors::sleep_insufficient::detect(&pool)
        .await
        .unwrap();
    assert_eq!(
        n, 0,
        "table vide → rows.len()=0 ≠ 3, retourne 0 sans panique"
    );
}

#[tokio::test]
async fn sleep_insufficient_only_one_record_returns_0() {
    let pool = crate::common::memory_pool().await;
    // 1 seule nuit (< 3 requises) → rows.len() != 3.
    sqlx::query(
        "INSERT INTO sleep_records (date, source, duration_total) VALUES (?1, 'withings', 18000)",
    )
    .bind(recent_date_only(0))
    .execute(&pool)
    .await
    .unwrap();
    let n = synk_lib::services::insights::detectors::sleep_insufficient::detect(&pool)
        .await
        .unwrap();
    assert_eq!(
        n, 0,
        "1 seule nuit → rows.len()=1 ≠ 3, retourne 0 sans panique"
    );
}

#[tokio::test]
async fn heart_rate_above_120_is_detected() {
    let pool = crate::common::memory_pool().await;
    for i in 1..=20 {
        sqlx::query(
            "INSERT INTO heart_records (timestamp, source, heart_rate) VALUES (?1, 'manual', 65)",
        )
        .bind(recent_iso(i))
        .execute(&pool)
        .await
        .unwrap();
    }
    sqlx::query(
        "INSERT INTO heart_records (timestamp, source, heart_rate) VALUES (?1, 'manual', 150)",
    )
    .bind(recent_iso(0))
    .execute(&pool)
    .await
    .unwrap();

    let n = synk_lib::services::insights::detectors::heart_rate_anomaly::detect(&pool)
        .await
        .unwrap();

    assert_eq!(n, 1, "150 bpm apres vingt mesures a 65 doit lever l'alerte");
}

#[tokio::test]
async fn backdated_row_inserted_last_is_not_the_latest() {
    let pool = crate::common::memory_pool().await;
    for i in 1..15 {
        sqlx::query(
            "INSERT INTO heart_records (timestamp, source, heart_rate) VALUES (?1, 'manual', 60)",
        )
        .bind(recent_iso(i))
        .execute(&pool)
        .await
        .unwrap();
    }
    sqlx::query(
        "INSERT INTO heart_records (timestamp, source, heart_rate) VALUES (?1, 'manual', 110)",
    )
    .bind(recent_iso(0))
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO heart_records (timestamp, source, heart_rate) VALUES (?1, 'manual', 60)",
    )
    .bind(recent_iso(20))
    .execute(&pool)
    .await
    .unwrap();

    let n = synk_lib::services::insights::detectors::heart_rate_anomaly::detect(&pool)
        .await
        .unwrap();

    assert_eq!(
        n, 1,
        "la mesure la plus recente (110) decide, pas la derniere inseree"
    );
}

#[tokio::test]
async fn failing_detector_is_reported_not_counted() {
    let pool = crate::common::memory_pool().await;
    sqlx::query("ALTER TABLE heart_records RENAME TO heart_records_hors_service")
        .execute(&pool)
        .await
        .unwrap();

    let report = run_generation(&pool).await;

    assert_eq!(report.failed, vec!["fréquence cardiaque".to_string()]);
    assert_eq!(report.generated, 0);
}

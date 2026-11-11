use sqlx::sqlite::SqlitePool;

#[derive(Debug)]
pub struct MigrationError {
    pub version: i64,
    pub source: sqlx::Error,
}

impl std::fmt::Display for MigrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Migration v{} failed: {}", self.version, self.source)
    }
}

impl std::error::Error for MigrationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

#[derive(Debug)]
pub enum InitError {
    Sqlx(sqlx::Error),
    Migration(MigrationError),
}

impl From<sqlx::Error> for InitError {
    fn from(e: sqlx::Error) -> Self {
        InitError::Sqlx(e)
    }
}

impl From<MigrationError> for InitError {
    fn from(e: MigrationError) -> Self {
        InitError::Migration(e)
    }
}

impl std::fmt::Display for InitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InitError::Sqlx(e) => write!(f, "{e}"),
            InitError::Migration(e) => write!(f, "{e}"),
        }
    }
}

const LAST_TOLERANT_VERSION: i64 = 16;

pub fn tolerated(version: i64, msg: &str) -> bool {
    version <= LAST_TOLERANT_VERSION
        && (msg.contains("duplicate column")
            || msg.contains("already another table")
            || msg.contains("already exists")
            || msg.contains("no such table"))
}

async fn rollback_logged(conn: &mut sqlx::SqliteConnection, version: i64) {
    if let Err(rollback) = sqlx::query("ROLLBACK").execute(&mut *conn).await {
        log::error!("[db] ROLLBACK de la migration {version} en echec : {rollback}");
    }
}

async fn foreign_key_violations(conn: &mut sqlx::SqliteConnection) -> Result<usize, sqlx::Error> {
    Ok(sqlx::query("PRAGMA foreign_key_check")
        .fetch_all(&mut *conn)
        .await?
        .len())
}

async fn no_new_violations(
    conn: &mut sqlx::SqliteConnection,
    before: usize,
) -> Result<(), sqlx::Error> {
    let after = foreign_key_violations(conn).await?;
    if after > before {
        return Err(sqlx::Error::Protocol(format!(
            "{} nouvelle(s) violation(s) de foreign key apres la migration, annulee",
            after - before
        )));
    }
    Ok(())
}

pub async fn run_versioned_migration(
    pool: &SqlitePool,
    version: i64,
    sql: &'static str,
) -> Result<(), MigrationError> {
    let applied: Option<(i64,)> =
        sqlx::query_as("SELECT version FROM schema_version WHERE version = ?1")
            .bind(version)
            .fetch_optional(pool)
            .await
            .map_err(|e| MigrationError { version, source: e })?;

    if applied.is_some() {
        return Ok(());
    }

    // Acquerir une seule connexion pour toute la migration : les PRAGMA SQLite
    // sont per-connection (ex: `legacy_alter_table` pour les rebuilds de table
    // sans propagation des FK references), donc executer tous les statements
    // sur la meme connexion garantit la coherence.
    let mut conn = pool
        .acquire()
        .await
        .map_err(|e| MigrationError { version, source: e })?;

    // `PRAGMA foreign_keys` est un no-op DANS une transaction : SQLite l'ignore
    // silencieusement. Les migrations qui reconstruisent une table (009, 016)
    // en dependent, donc les PRAGMA s'executent seuls, hors transaction, et
    // seul le reste est enveloppe. C'est la procedure officielle SQLite.
    let statements: Vec<&'static str> = sql
        .split(';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    // Le decoupage se fait sur ';', or les fichiers de migration font preceder
    // leur PRAGMA d'un bloc de commentaires sans ';' : le fragment commence donc
    // par '--' et non par 'PRAGMA'. Un simple starts_with laissait le PRAGMA
    // entrer dans la transaction, ou SQLite l'ignore en silence, et le DROP
    // TABLE de la migration 009 s'executait cles etrangeres actives, vidant par
    // cascade metrics, metric_targets et derived_metrics. Il faut donc sauter
    // les lignes de commentaire avant de tester le prefixe.
    let is_pragma = |s: &str| {
        s.lines()
            .map(|l| l.trim())
            .find(|l| !l.is_empty() && !l.starts_with("--"))
            .is_some_and(|l| l.to_ascii_uppercase().starts_with("PRAGMA"))
    };
    let leading = statements.iter().take_while(|s| is_pragma(s)).count();
    let trailing = statements
        .iter()
        .rev()
        .take_while(|s| is_pragma(s))
        .count()
        .min(statements.len() - leading);
    let body_end = statements.len() - trailing;

    for &statement in &statements[..leading] {
        sqlx::query(statement)
            .execute(&mut *conn)
            .await
            .map_err(|e| MigrationError { version, source: e })?;
    }

    let strict = version > LAST_TOLERANT_VERSION;
    let violations_before = if strict {
        foreign_key_violations(&mut conn)
            .await
            .map_err(|e| MigrationError { version, source: e })?
    } else {
        0
    };

    sqlx::query("BEGIN")
        .execute(&mut *conn)
        .await
        .map_err(|e| MigrationError { version, source: e })?;

    for &statement in &statements[leading..body_end] {
        if let Err(e) = sqlx::query(statement).execute(&mut *conn).await {
            let msg = e.to_string();
            if tolerated(version, &msg) {
                log::warn!(
                    "[db] migration {version} : instruction toleree ({msg}) : {}",
                    crate::utils::text::truncate_chars(statement, 80)
                );
            } else {
                rollback_logged(&mut conn, version).await;
                return Err(MigrationError { version, source: e });
            }
        }
    }

    // Le numero de version est ecrit DANS la transaction : un rollback ne doit
    // pas laisser une migration marquee appliquee alors qu'elle ne l'est pas.
    if let Err(e) = sqlx::query("INSERT OR IGNORE INTO schema_version (version) VALUES (?1)")
        .bind(version)
        .execute(&mut *conn)
        .await
    {
        rollback_logged(&mut conn, version).await;
        return Err(MigrationError { version, source: e });
    }

    if strict {
        if let Err(source) = no_new_violations(&mut conn, violations_before).await {
            rollback_logged(&mut conn, version).await;
            return Err(MigrationError { version, source });
        }
    }

    sqlx::query("COMMIT")
        .execute(&mut *conn)
        .await
        .map_err(|e| MigrationError { version, source: e })?;

    for &statement in &statements[body_end..] {
        sqlx::query(statement)
            .execute(&mut *conn)
            .await
            .map_err(|e| MigrationError { version, source: e })?;
    }

    Ok(())
}

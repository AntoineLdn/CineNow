use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::str::FromStr;

/// Crée le pool de connexions SQLite et applique les migrations.
///
/// `database_url` est de la forme "sqlite:///data/cinenow.db".
/// Le fichier est créé s'il n'existe pas (create_if_missing).
pub async fn init_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = sqlx::sqlite::SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        // WAL : meilleures performances en lecture/écriture concurrente.
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        // Applique les contraintes FOREIGN KEY (désactivées par défaut en SQLite).
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    // Applique les fichiers de migrations/ embarqués à la compilation.
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

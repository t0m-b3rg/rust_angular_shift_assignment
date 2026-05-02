use sqlx::sqlite::{SqlitePool, SqlitePoolOptions, SqliteConnectOptions};
use std::str::FromStr;

pub async fn init_db(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    // Create connection options with create_if_missing
    let connect_options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true);

    // Create database connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await?;

    // Create departments table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS departments (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            bereich TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )"
    )
    .execute(&pool)
    .await?;

    // Create employees table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS employees (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            rolle TEXT NOT NULL,
            qualifikation TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )"
    )
    .execute(&pool)
    .await?;

    // Create shifts table (shift templates)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS shifts (
            id TEXT PRIMARY KEY,
            typ TEXT NOT NULL,
            start_time TEXT NOT NULL,
            end_time TEXT NOT NULL,
            department_id TEXT NOT NULL,
            arbeitsstunden INTEGER NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(department_id) REFERENCES departments(id) ON DELETE CASCADE
        )"
    )
    .execute(&pool)
    .await?;

    // Create assigned_shifts table (shift assignments with references to templates)
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS assigned_shifts (
            id TEXT PRIMARY KEY,
            datum TEXT NOT NULL,
            arbeitsschichten_id TEXT NOT NULL,
            mitarbeiter_ids TEXT NOT NULL,
            state TEXT NOT NULL DEFAULT 'valid',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(arbeitsschichten_id) REFERENCES shifts(id) ON DELETE CASCADE
        )"
    )
    .execute(&pool)
    .await?;

    println!("[DATABASE] ✓ Database initialized successfully");

    Ok(pool)
}

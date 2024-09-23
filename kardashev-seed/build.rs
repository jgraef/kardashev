use std::path::PathBuf;

use sqlx::sqlite::SqliteConnectOptions;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let crate_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR not set"));

    let migrations_dir = crate_dir.join("migrations");
    let db_file = out_dir.join("schema.db");

    println!("cargo::rerun-if-changed={}", migrations_dir.display());

    // delete existing file
    if db_file.exists() {
        std::fs::remove_file(&db_file).unwrap_or_else(|_| {
            panic!(
                "Failed to delete existing database file: {}",
                db_file.display()
            )
        });
    }

    // create new database
    let options = SqliteConnectOptions::new()
        .filename(&db_file)
        .create_if_missing(true);
    let pool = sqlx::SqlitePool::connect_with(options)
        .await
        .unwrap_or_else(|_| panic!("Could not open database: {}", db_file.display()));
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .unwrap_or_else(|e| panic!("Failed to run migrations: {e}"));
    pool.close().await;

    println!(
        "cargo::rustc-env=DATABASE_URL=sqlite://{}",
        db_file.display()
    );
}

#[macro_use]
mod handlers;
mod models;
mod state;
mod db;
mod init_demo_db;

use axum::routing::{delete, get, post, put};
use axum::Router;
use tower_http::cors::CorsLayer;

use handlers::*;
use state::AppState;

#[tokio::main]
async fn main() {
    // Check if DEBUG mode is enabled
    let debug = std::env::var("DEBUG")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false);

    if debug {
        println!("[DEBUG] Debug mode enabled");
    }

    // Initialize database
    let db_url = "sqlite://simple_rust_api.db";
    let db = db::init_db(db_url)
        .await
        .expect("Failed to initialize database");

    // Initialize demo data if DEMO environment variable is set
    if std::env::var("DEMO").is_ok() {
        init_demo_db::init_demo_data(&db)
            .await
            .expect("Failed to initialize demo data");
    }

    let state = AppState { db, debug };

    // Configure CORS to allow requests from frontend
    let cors = CorsLayer::permissive();

    let router = Router::new()
        // Department routes
        .route("/departments", post(create_department))
        .route("/departments", get(list_departments))
        .route("/departments/{id}", get(get_department))
        .route("/departments/{id}", put(update_department))
        .route("/departments/{id}", delete(delete_department))
        // Employee routes
        .route("/employees", post(create_employee))
        .route("/employees", get(list_employees))
        .route("/employees/{id}", get(get_employee))
        .route("/employees/{id}", put(update_employee))
        .route("/employees/{id}", delete(delete_employee))
        // Shift routes
        .route("/shifts", post(create_shift))
        .route("/shifts", get(list_shifts))
        .route("/shifts/{id}", get(get_shift))
        .route("/shifts/{id}", put(update_shift))
        .route("/shifts/{id}", delete(delete_shift))
        .route("/departments/{department_id}/shifts", get(get_department_shifts))
        // Assigned Shift routes
        .route("/assigned-shifts", post(create_assigned_shift))
        .route("/assigned-shifts", get(list_assigned_shifts))
        .route("/assigned-shifts/init", post(init_assigned_shifts))
        .route("/assigned-shifts/{id}", get(get_assigned_shift))
        .route("/assigned-shifts/{id}", put(update_assigned_shift))
        .route("/assigned-shifts/{id}", delete(delete_assigned_shift))
        .route("/departments/{department_id}/assigned-shifts", get(get_department_assigned_shifts))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("\n╔════════════════════════════════════════╗");
    println!("║  🚀 Server running at http://127.0.0.1:3000  ║");
    println!("║  Press Ctrl+C to stop                ║");
    println!("╚════════════════════════════════════════╝\n");

    axum::serve(listener, router).await.unwrap();
}

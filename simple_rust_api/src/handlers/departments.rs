use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::models::*;
use crate::state::AppState;
use crate::debug_log;

// ==================== DEPARTMENT HANDLERS ====================

pub async fn create_department(
    State(state): State<AppState>,
    Json(req): Json<CreateDepartmentRequest>,
) -> Result<(StatusCode, Json<Department>), (StatusCode, String)> {
    debug_log!(state.debug, "[CREATE DEPARTMENT] Received request - name: {}, bereich: {}", req.name, req.bereich);
    
    let id = Uuid::new_v4().to_string();
    debug_log!(state.debug, "[CREATE DEPARTMENT] Generated ID: {}", id);

    let result = sqlx::query(
        "INSERT INTO departments (id, name, bereich) VALUES (?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.name)
    .bind(&req.bereich)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            // Fetch created_at from database
            let created_at: Option<String> = sqlx::query_scalar(
                "SELECT strftime('%Y-%m-%dT%H:%M:%SZ', created_at) FROM departments WHERE id = ?"
            )
            .bind(&id)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten();
            
            debug_log!(state.debug, "[CREATE DEPARTMENT] ✓ Department created successfully");
            let department = Department {
                id,
                name: req.name,
                bereich: req.bereich,
                created_at,
            };
            Ok((StatusCode::CREATED, Json(department)))
        }
        Err(e) => {
            debug_log!(state.debug, "[CREATE DEPARTMENT] ✗ Error: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

pub async fn list_departments(State(state): State<AppState>) -> Result<Json<Vec<Department>>, StatusCode> {
    let departments: Vec<Department> = sqlx::query_as::<_, Department>(
        "SELECT id, name, bereich, strftime('%Y-%m-%dT%H:%M:%SZ', created_at) as created_at FROM departments"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    debug_log!(state.debug, "[LIST DEPARTMENTS] Retrieved {} departments", departments.len());
    Ok(Json(departments))
}

pub async fn get_department(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Department>, StatusCode> {
    debug_log!(state.debug, "[GET DEPARTMENT] Searching for department ID: {}", id);
    
    let department: Option<Department> = sqlx::query_as::<_, Department>(
        "SELECT id, name, bereich, strftime('%Y-%m-%dT%H:%M:%SZ', created_at) as created_at FROM departments WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match department {
        Some(dept) => {
            debug_log!(state.debug, "[GET DEPARTMENT] ✓ Found department: {}", dept.name);
            Ok(Json(dept))
        }
        None => {
            debug_log!(state.debug, "[GET DEPARTMENT] ✗ Department not found");
            Err(StatusCode::NOT_FOUND)
        }
    }
}

pub async fn update_department(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateDepartmentRequest>,
) -> Result<Json<Department>, StatusCode> {
    debug_log!(state.debug, "[UPDATE DEPARTMENT] Updating department ID: {}", id);
    
    let current: Department = sqlx::query_as::<_, Department>(
        "SELECT id, name, bereich, strftime('%Y-%m-%dT%H:%M:%SZ', created_at) as created_at FROM departments WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let new_name = req.name.as_ref().unwrap_or(&current.name).clone();
    let new_bereich = req.bereich.as_ref().unwrap_or(&current.bereich).clone();

    let mut changes = Vec::new();
    if let Some(name) = &req.name {
        changes.push(format!("name: {} → {}", current.name, name));
    }
    if let Some(bereich) = &req.bereich {
        changes.push(format!("bereich: {} → {}", current.bereich, bereich));
    }

    sqlx::query("UPDATE departments SET name = ?, bereich = ? WHERE id = ?")
        .bind(&new_name)
        .bind(&new_bereich)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    debug_log!(state.debug, "[UPDATE DEPARTMENT] ✓ Changes applied: {}", changes.join(", "));

    Ok(Json(Department {
        id,
        name: new_name,
        bereich: new_bereich,
        created_at: current.created_at,
    }))
}

pub async fn delete_department(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> StatusCode {
    debug_log!(state.debug, "[DELETE DEPARTMENT] Deleting department ID: {}", id);
    
    let result = sqlx::query("DELETE FROM departments WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await;

    match result {
        Ok(result) if result.rows_affected() > 0 => {
            debug_log!(state.debug, "[DELETE DEPARTMENT] ✓ Department deleted");
            StatusCode::NO_CONTENT
        }
        Ok(_) => {
            debug_log!(state.debug, "[DELETE DEPARTMENT] ✗ Department not found");
            StatusCode::NOT_FOUND
        }
        Err(_) => {
            debug_log!(state.debug, "[DELETE DEPARTMENT] ✗ Error deleting department");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use sqlx::Row;

use crate::models::*;
use crate::state::AppState;
use crate::debug_log;

// ==================== EMPLOYEE HANDLERS ====================

pub async fn create_employee(
    State(state): State<AppState>,
    Json(req): Json<CreateEmployeeRequest>,
) -> Result<(StatusCode, Json<Employee>), (StatusCode, String)> {
    debug_log!(state.debug, "[CREATE EMPLOYEE] Received request - name: {}, rolle: {}, qualifikation: {}", req.name, req.rolle, req.qualifikation.as_str());
    
    let id = Uuid::new_v4().to_string();
    debug_log!(state.debug, "[CREATE EMPLOYEE] Generated ID: {}", id);

    let result = sqlx::query(
        "INSERT INTO employees (id, name, rolle, qualifikation) VALUES (?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.name)
    .bind(&req.rolle)
    .bind(req.qualifikation.as_str())
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            // Fetch created_at from database
            let created_at: Option<String> = sqlx::query_scalar(
                "SELECT strftime('%Y-%m-%dT%H:%M:%SZ', created_at) FROM employees WHERE id = ?"
            )
            .bind(&id)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten();
            
            debug_log!(state.debug, "[CREATE EMPLOYEE] ✓ Employee created successfully");
            let employee = Employee {
                id,
                name: req.name,
                rolle: req.rolle,
                qualifikation: req.qualifikation,
                created_at,
            };
            Ok((StatusCode::CREATED, Json(employee)))
        }
        Err(e) => {
            debug_log!(state.debug, "[CREATE EMPLOYEE] ✗ Error: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        }
    }
}

pub async fn list_employees(State(state): State<AppState>) -> Result<Json<Vec<Employee>>, StatusCode> {
    let rows = sqlx::query(
        "SELECT id, name, rolle, qualifikation, strftime('%Y-%m-%dT%H:%M:%SZ', created_at) as created_at FROM employees"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let employees: Vec<Employee> = rows.into_iter().map(|row| {
        let qual_str: String = row.get("qualifikation");
        let qualifikation = Qualifikation::from_str(&qual_str).unwrap_or(Qualifikation::Pflegehelfer);
        Employee {
            id: row.get("id"),
            name: row.get("name"),
            rolle: row.get("rolle"),
            qualifikation,
            created_at: row.get("created_at"),
        }
    }).collect();

    debug_log!(state.debug, "[LIST EMPLOYEES] Retrieved {} employees", employees.len());
    Ok(Json(employees))
}

pub async fn get_employee(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Employee>, StatusCode> {
    debug_log!(state.debug, "[GET EMPLOYEE] Searching for employee ID: {}", id);
    
    let row = sqlx::query(
        "SELECT id, name, rolle, qualifikation, strftime('%Y-%m-%dT%H:%M:%SZ', created_at) as created_at FROM employees WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        Some(row) => {
            let qual_str: String = row.get("qualifikation");
            let qualifikation = Qualifikation::from_str(&qual_str).unwrap_or(Qualifikation::Pflegehelfer);
            
            let emp = Employee {
                id: row.get("id"),
                name: row.get("name"),
                rolle: row.get("rolle"),
                qualifikation,
                created_at: row.get("created_at"),
            };
            debug_log!(state.debug, "[GET EMPLOYEE] ✓ Found employee: {}", emp.name);
            Ok(Json(emp))
        }
        None => {
            debug_log!(state.debug, "[GET EMPLOYEE] ✗ Employee not found");
            Err(StatusCode::NOT_FOUND)
        }
    }
}

pub async fn update_employee(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateEmployeeRequest>,
) -> Result<Json<Employee>, StatusCode> {
    debug_log!(state.debug, "[UPDATE EMPLOYEE] Updating employee ID: {}", id);
    
    let row = sqlx::query(
        "SELECT id, name, rolle, qualifikation, strftime('%Y-%m-%dT%H:%M:%SZ', created_at) as created_at FROM employees WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let current_name: String = row.get("name");
    let current_rolle: String = row.get("rolle");
    let current_qual_str: String = row.get("qualifikation");
    let current_qualifikation = Qualifikation::from_str(&current_qual_str).unwrap_or(Qualifikation::Pflegehelfer);
    let created_at: Option<String> = row.get("created_at");

    let new_name = req.name.as_ref().unwrap_or(&current_name).clone();
    let new_rolle = req.rolle.as_ref().unwrap_or(&current_rolle).clone();
    let new_qualifikation = req.qualifikation.as_ref().unwrap_or(&current_qualifikation).clone();

    let mut changes = Vec::new();
    if let Some(name) = &req.name {
        changes.push(format!("name: {} → {}", current_name, name));
    }
    if let Some(rolle) = &req.rolle {
        changes.push(format!("rolle: {} → {}", current_rolle, rolle));
    }
    if let Some(qual) = &req.qualifikation {
        changes.push(format!("qualifikation: {} → {}", current_qualifikation.as_str(), qual.as_str()));
    }

    sqlx::query("UPDATE employees SET name = ?, rolle = ?, qualifikation = ? WHERE id = ?")
        .bind(&new_name)
        .bind(&new_rolle)
        .bind(new_qualifikation.as_str())
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    debug_log!(state.debug, "[UPDATE EMPLOYEE] ✓ Changes applied: {}", changes.join(", "));

    Ok(Json(Employee {
        id,
        name: new_name,
        rolle: new_rolle,
        qualifikation: new_qualifikation,
        created_at,
    }))
}

pub async fn delete_employee(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> StatusCode {
    debug_log!(state.debug, "[DELETE EMPLOYEE] Deleting employee ID: {}", id);
    
    let result = sqlx::query("DELETE FROM employees WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await;

    match result {
        Ok(result) if result.rows_affected() > 0 => {
            debug_log!(state.debug, "[DELETE EMPLOYEE] ✓ Employee deleted");
            StatusCode::NO_CONTENT
        }
        Ok(_) => {
            debug_log!(state.debug, "[DELETE EMPLOYEE] ✗ Employee not found");
            StatusCode::NOT_FOUND
        }
        Err(_) => {
            debug_log!(state.debug, "[DELETE EMPLOYEE] ✗ Error deleting employee");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

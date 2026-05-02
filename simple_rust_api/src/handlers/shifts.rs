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

// ==================== SHIFT HANDLERS ====================

pub async fn create_shift(
    State(state): State<AppState>,
    Json(req): Json<CreateShiftRequest>,
) -> Result<(StatusCode, Json<Shift>), StatusCode> {
    let typ_str = req.typ.as_str();
    debug_log!(state.debug, "[CREATE SHIFT] Received request - typ: {}", typ_str);
    
    // Verify department exists
    let dept_exists: bool = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM departments WHERE id = ?)"
    )
    .bind(&req.department_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !dept_exists {
        debug_log!(state.debug, "[CREATE SHIFT] ✗ Department not found");
        return Err(StatusCode::BAD_REQUEST);
    }

    let id = Uuid::new_v4().to_string();
    debug_log!(state.debug, "[CREATE SHIFT] ✓ Department verified, generated shift ID: {}", id);

    let result = sqlx::query(
        "INSERT INTO shifts (id, typ, start_time, end_time, department_id, arbeitsstunden) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(typ_str)
    .bind(&req.start_time)
    .bind(&req.end_time)
    .bind(&req.department_id)
    .bind(req.arbeitsstunden)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            // Fetch created_at from database
            let created_at: Option<String> = sqlx::query_scalar(
                "SELECT strftime('%Y-%m-%dT%H:%M:%SZ', created_at) FROM shifts WHERE id = ?"
            )
            .bind(&id)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten();
            
            debug_log!(state.debug, "[CREATE SHIFT] ✓ Shift template created successfully");
            let shift = Shift {
                id,
                datum: String::new(),
                typ: typ_str.to_string(),
                start_time: req.start_time,
                end_time: req.end_time,
                department_id: req.department_id,
                arbeitsstunden: req.arbeitsstunden,
                created_at,
            };
            Ok((StatusCode::CREATED, Json(shift)))
        }
        Err(e) => {
            debug_log!(state.debug, "[CREATE SHIFT] ✗ Error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn list_shifts(State(state): State<AppState>) -> Result<Json<Vec<Shift>>, StatusCode> {
    let rows = sqlx::query(
        "SELECT id, typ, start_time, end_time, department_id, arbeitsstunden, strftime('%Y-%m-%dT%H:%M:%SZ', created_at) as created_at FROM shifts"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let shifts: Vec<Shift> = rows.into_iter().map(|row| Shift {
        id: row.get("id"),
        datum: String::new(),
        typ: row.get("typ"),
        start_time: row.get("start_time"),
        end_time: row.get("end_time"),
        department_id: row.get("department_id"),
        arbeitsstunden: row.get("arbeitsstunden"),
        created_at: row.get("created_at"),
    }).collect();

    debug_log!(state.debug, "[LIST SHIFTS] Retrieved {} shifts", shifts.len());
    Ok(Json(shifts))
}

pub async fn get_shift(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Shift>, StatusCode> {
    debug_log!(state.debug, "[GET SHIFT] Searching for shift ID: {}", id);
    
    let row = sqlx::query(
        "SELECT id, typ, start_time, end_time, department_id, arbeitsstunden, strftime('%Y-%m-%dT%H:%M:%SZ', created_at) as created_at FROM shifts WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        Some(row) => {
            let shift = Shift {
                id: row.get("id"),
                datum: String::new(),
                typ: row.get("typ"),
                start_time: row.get("start_time"),
                end_time: row.get("end_time"),
                department_id: row.get("department_id"),
                arbeitsstunden: row.get("arbeitsstunden"),
                created_at: row.get("created_at"),
            };
            debug_log!(state.debug, "[GET SHIFT] ✓ Found shift: '{}'", shift.typ);
            Ok(Json(shift))
        }
        None => {
            debug_log!(state.debug, "[GET SHIFT] ✗ Shift not found");
            Err(StatusCode::NOT_FOUND)
        }
    }
}

pub async fn update_shift(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateShiftRequest>,
) -> Result<Json<Shift>, StatusCode> {
    debug_log!(state.debug, "[UPDATE SHIFT] Updating shift ID: {}", id);
    
    let current_row = sqlx::query(
        "SELECT id, typ, start_time, end_time, department_id, arbeitsstunden, strftime('%Y-%m-%dT%H:%M:%SZ', created_at) as created_at FROM shifts WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let current_typ: String = current_row.get("typ");
    let current_start_time: String = current_row.get("start_time");
    let current_end_time: String = current_row.get("end_time");
    let current_department_id: String = current_row.get("department_id");
    let current_arbeitsstunden: i32 = current_row.get("arbeitsstunden");
    let created_at: Option<String> = current_row.get("created_at");

    let new_typ = req.typ.as_ref().map(|t| t.as_str()).unwrap_or(&current_typ).to_string();
    let new_start_time = req.start_time.as_ref().unwrap_or(&current_start_time).clone();
    let new_end_time = req.end_time.as_ref().unwrap_or(&current_end_time).clone();

    let mut changes = Vec::new();
    if let Some(typ) = &req.typ {
        changes.push(format!("typ: {} → {}", current_typ, typ.as_str()));
    }
    if let Some(start_time) = &req.start_time {
        changes.push(format!("start_time: {} → {}", current_start_time, start_time));
    }
    if let Some(end_time) = &req.end_time {
        changes.push(format!("end_time: {} → {}", current_end_time, end_time));
    }

    sqlx::query("UPDATE shifts SET typ = ?, start_time = ?, end_time = ? WHERE id = ?")
        .bind(&new_typ)
        .bind(&new_start_time)
        .bind(&new_end_time)
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    debug_log!(state.debug, "[UPDATE SHIFT] ✓ Changes applied: {}", changes.join(", "));
    Ok(Json(Shift {
        id,
        datum: String::new(),
        typ: new_typ,
        start_time: new_start_time,
        end_time: new_end_time,
        department_id: current_department_id,
        arbeitsstunden: current_arbeitsstunden,
        created_at,
    }))
}

pub async fn delete_shift(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> StatusCode {
    debug_log!(state.debug, "[DELETE SHIFT] Deleting shift ID: {}", id);
    
    let result = sqlx::query("DELETE FROM shifts WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await;

    match result {
        Ok(result) if result.rows_affected() > 0 => {
            debug_log!(state.debug, "[DELETE SHIFT] ✓ Shift deleted");
            StatusCode::NO_CONTENT
        }
        Ok(_) => {
            debug_log!(state.debug, "[DELETE SHIFT] ✗ Shift not found");
            StatusCode::NOT_FOUND
        }
        Err(_) => {
            debug_log!(state.debug, "[DELETE SHIFT] ✗ Error deleting shift");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn get_department_shifts(
    State(state): State<AppState>,
    Path(department_id): Path<String>,
) -> Result<Json<Vec<Shift>>, StatusCode> {
    debug_log!(state.debug, "[GET DEPARTMENT SHIFTS] Fetching shifts for department: {}", department_id);
    
    let rows = sqlx::query(
        "SELECT id, typ, start_time, end_time, department_id, arbeitsstunden, strftime('%Y-%m-%dT%H:%M:%SZ', created_at) as created_at FROM shifts WHERE department_id = ?"
    )
    .bind(&department_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let shifts: Vec<Shift> = rows.into_iter().map(|row| Shift {
        id: row.get("id"),
        datum: String::new(),
        typ: row.get("typ"),
        start_time: row.get("start_time"),
        end_time: row.get("end_time"),
        department_id: row.get("department_id"),
        arbeitsstunden: row.get("arbeitsstunden"),
        created_at: row.get("created_at"),
    }).collect();

    debug_log!(state.debug, "[GET DEPARTMENT SHIFTS] ✓ Found {} shifts for department", shifts.len());
    Ok(Json(shifts))
}

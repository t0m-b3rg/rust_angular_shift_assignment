use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    response::{IntoResponse, Response},
};
use uuid::Uuid;
use sqlx::Row;

use crate::models::*;
use crate::state::AppState;
use crate::debug_log;
use crate::handlers::common::{times_overlap, time_to_minutes, determine_assigned_shift_state};

// Custom error type that returns JSON with descriptive message
pub struct ApiErrorResponse {
    pub status: StatusCode,
    pub body: ApiError,
}

impl IntoResponse for ApiErrorResponse {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}

/// Helper to compute UTC timestamp from date and time (e.g., "2024-01-15" + "06:00" -> "2024-01-15T06:00:00Z")
fn compute_utc_timestamp(datum: &str, time: &str) -> String {
    format!("{}T{}:00Z", datum, time)
}

// ==================== ASSIGNED SHIFT HANDLERS ====================

pub async fn create_assigned_shift(
    State(state): State<AppState>,
    Json(req): Json<CreateAssignedShiftRequest>,
) -> Result<(StatusCode, Json<AssignedShift>), ApiErrorResponse> {
    debug_log!(state.debug, "[CREATE ASSIGNED SHIFT] Received request - datum: {}, arbeitsschichten_id: {}", req.datum, req.arbeitsschichten_id);
    
    // Verify shift template exists and get its times and work hours
    let template_row = sqlx::query(
        "SELECT s.id, s.typ, s.start_time, s.end_time, s.arbeitsstunden, s.department_id, d.name as department_name 
         FROM shifts s JOIN departments d ON s.department_id = d.id WHERE s.id = ?"
    )
    .bind(&req.arbeitsschichten_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ApiErrorResponse {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        body: ApiError {
            error: "database_error".to_string(),
            message: format!("Database error: {}", e),
            details: None,
        },
    })?
    .ok_or_else(|| {
        debug_log!(state.debug, "[CREATE ASSIGNED SHIFT] ✗ Shift template not found");
        ApiErrorResponse {
            status: StatusCode::BAD_REQUEST,
            body: ApiError {
                error: "shift_not_found".to_string(),
                message: format!("Shift template '{}' not found", req.arbeitsschichten_id),
                details: None,
            },
        }
    })?;

    let new_start_time: String = template_row.get("start_time");
    let new_end_time: String = template_row.get("end_time");
    let new_shift_typ: String = template_row.get("typ");
    let new_department_name: String = template_row.get("department_name");
    let template_arbeitsstunden: i32 = template_row.get("arbeitsstunden");

    debug_log!(state.debug, "[CREATE ASSIGNED SHIFT] ✓ Template verified: {} ({} - {}) in {} - {} hours required", req.arbeitsschichten_id, new_start_time, new_end_time, new_department_name, template_arbeitsstunden);

    // Check for scheduling conflicts for each employee
    for emp_id in &req.mitarbeiter_ids {
        debug_log!(state.debug, "[CREATE ASSIGNED SHIFT] Checking conflicts for employee: {}", emp_id);
        
        // Get employee name for better error messages
        let emp_name: String = sqlx::query_scalar("SELECT name FROM employees WHERE id = ?")
            .bind(emp_id)
            .fetch_optional(&state.db)
            .await
            .map_err(|_| ApiErrorResponse {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                body: ApiError {
                    error: "database_error".to_string(),
                    message: "Failed to fetch employee details".to_string(),
                    details: None,
                },
            })?
            .unwrap_or_else(|| format!("Unknown ({})", emp_id));
        
        // Calculate the previous day for checking overnight shifts
        let prev_datum = chrono::NaiveDate::parse_from_str(&req.datum, "%Y-%m-%d")
            .ok()
            .and_then(|d| d.pred_opt())
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_default();
        
        let like_pattern = format!("%\"{}\"%", emp_id);
        
        // Find all assigned shifts for this employee on the same date OR overnight shifts from the previous day
        let conflicting_rows = sqlx::query(
            "SELECT a.id, a.datum, a.arbeitsschichten_id FROM assigned_shifts a 
             JOIN shifts s ON a.arbeitsschichten_id = s.id
             WHERE (a.datum = ? OR (a.datum = ? AND s.end_time < s.start_time)) 
             AND a.mitarbeiter_ids LIKE ?"
        )
        .bind(&req.datum)
        .bind(&prev_datum)  // Also check previous day for overnight shifts
        .bind(&like_pattern)
        .fetch_all(&state.db)
        .await
        .map_err(|_| ApiErrorResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: ApiError {
                error: "database_error".to_string(),
                message: "Failed to check for conflicts".to_string(),
                details: None,
            },
        })?;

        for conflict_row in conflicting_rows {
            let other_shift_id: String = conflict_row.get("arbeitsschichten_id");
            let other_datum: String = conflict_row.get("datum");
            
            // Get the other shift's template details including department
            let other_template = sqlx::query(
                "SELECT s.typ, s.start_time, s.end_time, d.name as department_name 
                 FROM shifts s JOIN departments d ON s.department_id = d.id WHERE s.id = ?"
            )
            .bind(&other_shift_id)
            .fetch_optional(&state.db)
            .await
            .map_err(|_| ApiErrorResponse {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                body: ApiError {
                    error: "database_error".to_string(),
                    message: "Failed to fetch shift details".to_string(),
                    details: None,
                },
            })?
            .ok_or_else(|| ApiErrorResponse {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                body: ApiError {
                    error: "data_integrity_error".to_string(),
                    message: "Referenced shift template not found".to_string(),
                    details: None,
                },
            })?;

            let other_shift_typ: String = other_template.get("typ");
            let other_start_time: String = other_template.get("start_time");
            let other_end_time: String = other_template.get("end_time");
            let other_department_name: String = other_template.get("department_name");
            
            // Determine if this is a cross-day conflict (overnight shift from previous day)
            let is_cross_day_conflict = other_datum != req.datum;

            // Check if times overlap - use cross-day aware comparison
            let has_overlap = if is_cross_day_conflict {
                // The other shift is an overnight shift from the previous day
                // Check if the new shift starts before the overnight shift ends
                let other_end_mins = time_to_minutes(&other_end_time);
                let new_start_mins = time_to_minutes(&new_start_time);
                let new_end_mins = time_to_minutes(&new_end_time);
                let is_new_overnight = new_end_mins < new_start_mins;
                
                // New shift overlaps if it starts before the overnight shift ends
                // Or if the new shift is also overnight (they'd definitely overlap)
                new_start_mins < other_end_mins || is_new_overnight
            } else {
                // Same day - use normal overlap detection
                times_overlap(&new_start_time, &new_end_time, &other_start_time, &other_end_time)
            };
            
            if has_overlap {
                debug_log!(state.debug, "[CREATE ASSIGNED SHIFT] ✗ Conflict: {} is already scheduled for {} shift in {} ({}-{}) on {}", 
                    emp_id, other_shift_typ, other_department_name, other_start_time, other_end_time, req.datum);
                return Err(ApiErrorResponse {
                    status: StatusCode::CONFLICT,
                    body: ApiError {
                        error: "scheduling_conflict".to_string(),
                        message: format!(
                            "Employee '{}' cannot be assigned to {} shift ({}-{}) in '{}' on {} because they are already scheduled for {} shift ({}-{}) in '{}'",
                            emp_name, new_shift_typ, new_start_time, new_end_time, new_department_name, req.datum,
                            other_shift_typ, other_start_time, other_end_time, other_department_name
                        ),
                        details: Some(ConflictDetails {
                            employee_id: emp_id.clone(),
                            employee_name: emp_name,
                            date: req.datum.clone(),
                            conflicting_shift: ConflictingShiftInfo {
                                shift_id: other_shift_id,
                                shift_type: other_shift_typ,
                                start_time: other_start_time,
                                end_time: other_end_time,
                                department_name: Some(other_department_name),
                            },
                            requested_shift: ShiftTimeInfo {
                                start_time: new_start_time.clone(),
                                end_time: new_end_time.clone(),
                            },
                        }),
                    },
                });
            }
        }
    }

    debug_log!(state.debug, "[CREATE ASSIGNED SHIFT] ✓ No scheduling conflicts detected");

    let id = Uuid::new_v4().to_string();
    let mitarbeiter_json = serde_json::to_string(&req.mitarbeiter_ids)
        .unwrap_or_else(|_| "[]".to_string());
    
    // Determine state based on employees and work hours using helper function
    let shift_state = determine_assigned_shift_state(
        req.mitarbeiter_ids.len(),
        template_arbeitsstunden,
        req.state,
        state.debug,
    );

    debug_log!(state.debug, "[CREATE ASSIGNED SHIFT] Generated assignment ID: {}", id);

    let result = sqlx::query(
        "INSERT INTO assigned_shifts (id, datum, arbeitsschichten_id, mitarbeiter_ids, state) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.datum)
    .bind(&req.arbeitsschichten_id)
    .bind(&mitarbeiter_json)
    .bind(shift_state.as_str())
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => {
            debug_log!(state.debug, "[CREATE ASSIGNED SHIFT] ✓ Assigned shift created successfully");
            
            // Log assigned employees
            if !req.mitarbeiter_ids.is_empty() {
                let mut emp_names = Vec::new();
                for emp_id in &req.mitarbeiter_ids {
                    let name = sqlx::query_scalar::<_, String>("SELECT name FROM employees WHERE id = ?")
                        .bind(emp_id)
                        .fetch_optional(&state.db)
                        .await
                        .ok()
                        .flatten()
                        .unwrap_or_else(|| emp_id.clone());
                    emp_names.push(name);
                }
                debug_log!(state.debug, "[CREATE ASSIGNED SHIFT] 📋 Assigned {} employees to {} shift ({}-{}) in '{}' on {}: {}",
                    req.mitarbeiter_ids.len(), new_shift_typ, new_start_time, new_end_time, new_department_name, req.datum,
                    emp_names.join(", "));
            } else {
                debug_log!(state.debug, "[CREATE ASSIGNED SHIFT] 📋 Created unassigned {} shift ({}-{}) in '{}' on {}",
                    new_shift_typ, new_start_time, new_end_time, new_department_name, req.datum);
            }
            
            // Fetch created_at from database
            let created_at: Option<String> = sqlx::query_scalar(
                "SELECT strftime('%Y-%m-%dT%H:%M:%SZ', created_at) FROM assigned_shifts WHERE id = ?"
            )
            .bind(&id)
            .fetch_optional(&state.db)
            .await
            .ok()
            .flatten();
            
            let assigned_shift = AssignedShift {
                id,
                datum: req.datum.clone(),
                arbeitsschichten_id: req.arbeitsschichten_id,
                mitarbeiter_ids: req.mitarbeiter_ids,
                state: shift_state,
                start_utc: Some(compute_utc_timestamp(&req.datum, &new_start_time)),
                end_utc: Some(compute_utc_timestamp(&req.datum, &new_end_time)),
                created_at,
            };
            Ok((StatusCode::CREATED, Json(assigned_shift)))
        }
        Err(e) => {
            debug_log!(state.debug, "[CREATE ASSIGNED SHIFT] ✗ Error: {}", e);
            Err(ApiErrorResponse {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                body: ApiError {
                    error: "database_error".to_string(),
                    message: format!("Failed to create assigned shift: {}", e),
                    details: None,
                },
            })
        }
    }
}

pub async fn list_assigned_shifts(State(state): State<AppState>) -> Result<Json<Vec<AssignedShift>>, StatusCode> {
    debug_log!(state.debug, "[LIST ASSIGNED SHIFTS] Fetching all assigned shifts");
    
    // Query with JOIN to get shift template times for UTC timestamp computation
    let rows = sqlx::query(
        "SELECT a.id, a.datum, a.arbeitsschichten_id, a.mitarbeiter_ids, a.state, 
                s.start_time, s.end_time,
                strftime('%Y-%m-%dT%H:%M:%SZ', a.created_at) as created_at
         FROM assigned_shifts a
         JOIN shifts s ON a.arbeitsschichten_id = s.id"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let assigned_shifts: Vec<AssignedShift> = rows.into_iter().map(|row| {
        let mitarbeiter_json: String = row.get("mitarbeiter_ids");
        let mitarbeiter_ids: Vec<String> = serde_json::from_str(&mitarbeiter_json)
            .unwrap_or_default();
        let state_str: String = row.get("state");
        let shift_state = AssignedShiftState::from_str(&state_str).unwrap_or(AssignedShiftState::Valid);
        let datum: String = row.get("datum");
        let start_time: String = row.get("start_time");
        let end_time: String = row.get("end_time");
        
        AssignedShift {
            id: row.get("id"),
            datum: datum.clone(),
            arbeitsschichten_id: row.get("arbeitsschichten_id"),
            mitarbeiter_ids,
            state: shift_state,
            start_utc: Some(compute_utc_timestamp(&datum, &start_time)),
            end_utc: Some(compute_utc_timestamp(&datum, &end_time)),
            created_at: row.get("created_at"),
        }
    }).collect();

    debug_log!(state.debug, "[LIST ASSIGNED SHIFTS] Retrieved {} assigned shifts", assigned_shifts.len());
    Ok(Json(assigned_shifts))
}

pub async fn get_assigned_shift(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AssignedShift>, StatusCode> {
    debug_log!(state.debug, "[GET ASSIGNED SHIFT] Searching for assigned shift ID: {}", id);
    
    let row = sqlx::query(
        "SELECT a.id, a.datum, a.arbeitsschichten_id, a.mitarbeiter_ids, a.state,
                s.start_time, s.end_time,
                strftime('%Y-%m-%dT%H:%M:%SZ', a.created_at) as created_at
         FROM assigned_shifts a
         JOIN shifts s ON a.arbeitsschichten_id = s.id
         WHERE a.id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        Some(row) => {
            let mitarbeiter_json: String = row.get("mitarbeiter_ids");
            let mitarbeiter_ids: Vec<String> = serde_json::from_str(&mitarbeiter_json)
                .unwrap_or_default();
            let state_str: String = row.get("state");
            let shift_state = AssignedShiftState::from_str(&state_str).unwrap_or(AssignedShiftState::Valid);
            let datum: String = row.get("datum");
            let start_time: String = row.get("start_time");
            let end_time: String = row.get("end_time");
            
            debug_log!(state.debug, "[GET ASSIGNED SHIFT] ✓ Found assigned shift");
            Ok(Json(AssignedShift {
                id: row.get("id"),
                datum: datum.clone(),
                arbeitsschichten_id: row.get("arbeitsschichten_id"),
                mitarbeiter_ids,
                state: shift_state,
                start_utc: Some(compute_utc_timestamp(&datum, &start_time)),
                end_utc: Some(compute_utc_timestamp(&datum, &end_time)),
                created_at: row.get("created_at"),
            }))
        }
        None => {
            debug_log!(state.debug, "[GET ASSIGNED SHIFT] ✗ Assigned shift not found");
            Err(StatusCode::NOT_FOUND)
        }
    }
}

pub async fn update_assigned_shift(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateAssignedShiftRequest>,
) -> Result<Json<AssignedShift>, ApiErrorResponse> {
    debug_log!(state.debug, "[UPDATE ASSIGNED SHIFT] Updating assigned shift ID: {}", id);
    
    let row = sqlx::query(
        "SELECT id, datum, arbeitsschichten_id, mitarbeiter_ids, state FROM assigned_shifts WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ApiErrorResponse {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        body: ApiError {
            error: "database_error".to_string(),
            message: format!("Database error: {}", e),
            details: None,
        },
    })?
    .ok_or_else(|| ApiErrorResponse {
        status: StatusCode::NOT_FOUND,
        body: ApiError {
            error: "not_found".to_string(),
            message: format!("Assigned shift '{}' not found", id),
            details: None,
        },
    })?;

    let current_datum: String = row.get("datum");
    let current_arbeitsschichten_id: String = row.get("arbeitsschichten_id");
    let current_mitarbeiter_json: String = row.get("mitarbeiter_ids");
    let current_mitarbeiter_ids: Vec<String> = serde_json::from_str(&current_mitarbeiter_json)
        .unwrap_or_default();
    let current_state_str: String = row.get("state");
    let current_state = AssignedShiftState::from_str(&current_state_str).unwrap_or(AssignedShiftState::Valid);

    let new_datum = req.datum.as_ref().unwrap_or(&current_datum).clone();
    let mitarbeiter_changed = req.mitarbeiter_ids.is_some();
    let datum_changed = req.datum.is_some();
    
    let new_mitarbeiter_ids = req.mitarbeiter_ids.unwrap_or(current_mitarbeiter_ids.clone());

    // Get shift template details for conflict checking and work hours validation
    let template_row = sqlx::query(
        "SELECT s.typ, s.start_time, s.end_time, s.arbeitsstunden, d.name as department_name 
         FROM shifts s JOIN departments d ON s.department_id = d.id WHERE s.id = ?"
    )
    .bind(&current_arbeitsschichten_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| ApiErrorResponse {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        body: ApiError {
            error: "database_error".to_string(),
            message: format!("Failed to fetch shift template: {}", e),
            details: None,
        },
    })?;

    let shift_typ: String = template_row.get("typ");
    let start_time: String = template_row.get("start_time");
    let end_time: String = template_row.get("end_time");
    let department_name: String = template_row.get("department_name");
    let template_arbeitsstunden: i32 = template_row.get("arbeitsstunden");

    // Check for conflicts with newly added employees or on new date
    let employees_to_check: Vec<String> = if mitarbeiter_changed {
        // If employees changed, check only NEW employees
        new_mitarbeiter_ids.iter()
            .filter(|e| !current_mitarbeiter_ids.contains(e))
            .cloned()
            .collect()
    } else if datum_changed {
        // If only date changed, check all current employees
        new_mitarbeiter_ids.clone()
    } else {
        // No changes affecting conflicts
        vec![]
    };

    for emp_id in employees_to_check {
        debug_log!(state.debug, "[UPDATE ASSIGNED SHIFT] Checking conflicts for employee: {}", emp_id);
        
        // Get employee name for better error messages
        let emp_name: String = sqlx::query_scalar("SELECT name FROM employees WHERE id = ?")
            .bind(&emp_id)
            .fetch_optional(&state.db)
            .await
            .map_err(|_| ApiErrorResponse {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                body: ApiError {
                    error: "database_error".to_string(),
                    message: "Failed to fetch employee details".to_string(),
                    details: None,
                },
            })?
            .unwrap_or_else(|| format!("Unknown ({})", emp_id));
        
        // Calculate the previous day for checking overnight shifts
        let prev_datum = chrono::NaiveDate::parse_from_str(&new_datum, "%Y-%m-%d")
            .ok()
            .and_then(|d| d.pred_opt())
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_default();
        
        let like_pattern = format!("%\"{}\"%", emp_id);
        debug_log!(state.debug, "[UPDATE ASSIGNED SHIFT] Searching with LIKE pattern: {} on dates {} or overnight from {}", like_pattern, new_datum, prev_datum);
        
        // Find all assigned shifts for this employee on the same date OR overnight shifts from the previous day
        let conflicting_rows = sqlx::query(
            "SELECT a.id, a.datum, a.arbeitsschichten_id FROM assigned_shifts a 
             JOIN shifts s ON a.arbeitsschichten_id = s.id
             WHERE (a.datum = ? OR (a.datum = ? AND s.end_time < s.start_time)) 
             AND a.id != ? AND a.mitarbeiter_ids LIKE ?"
        )
        .bind(&new_datum)
        .bind(&prev_datum)  // Also check previous day for overnight shifts
        .bind(&id)  // Exclude the current shift from conflict check
        .bind(&like_pattern)
        .fetch_all(&state.db)
        .await
        .map_err(|_| ApiErrorResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: ApiError {
                error: "database_error".to_string(),
                message: "Failed to check for conflicts".to_string(),
                details: None,
            },
        })?;

        debug_log!(state.debug, "[UPDATE ASSIGNED SHIFT] Found {} potential conflicting shifts for {}", conflicting_rows.len(), emp_id);

        for conflict_row in conflicting_rows {
            let other_shift_id: String = conflict_row.get("arbeitsschichten_id");
            let other_datum: String = conflict_row.get("datum");
            let other_assigned_id: String = conflict_row.get("id");
            
            debug_log!(state.debug, "[UPDATE ASSIGNED SHIFT] Checking overlap with assigned_shift {} (shift template {}) on {}", 
                other_assigned_id, other_shift_id, other_datum);
            
            // Get the other shift's template details including department
            let other_template = sqlx::query(
                "SELECT s.typ, s.start_time, s.end_time, d.name as department_name 
                 FROM shifts s JOIN departments d ON s.department_id = d.id WHERE s.id = ?"
            )
            .bind(&other_shift_id)
            .fetch_optional(&state.db)
            .await
            .map_err(|_| ApiErrorResponse {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                body: ApiError {
                    error: "database_error".to_string(),
                    message: "Failed to fetch conflicting shift details".to_string(),
                    details: None,
                },
            })?
            .ok_or_else(|| ApiErrorResponse {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                body: ApiError {
                    error: "data_integrity_error".to_string(),
                    message: "Referenced shift template not found".to_string(),
                    details: None,
                },
            })?;

            let other_shift_typ: String = other_template.get("typ");
            let other_start_time: String = other_template.get("start_time");
            let other_end_time: String = other_template.get("end_time");
            let other_department_name: String = other_template.get("department_name");
            
            debug_log!(state.debug, "[UPDATE ASSIGNED SHIFT] Other shift: {} ({}-{}) in {}", 
                other_shift_typ, other_start_time, other_end_time, other_department_name);
            debug_log!(state.debug, "[UPDATE ASSIGNED SHIFT] Current shift: {} ({}-{}) in {}", 
                shift_typ, start_time, end_time, department_name);
            
            // Determine if this is a cross-day conflict (overnight shift from previous day)
            let is_cross_day_conflict = other_datum != new_datum;
            debug_log!(state.debug, "[UPDATE ASSIGNED SHIFT] Cross-day conflict: {}", is_cross_day_conflict);

            // Check if times overlap - use cross-day aware comparison
            let has_overlap = if is_cross_day_conflict {
                // The other shift is an overnight shift from the previous day
                // Check if the new shift starts before the overnight shift ends
                let other_end_mins = time_to_minutes(&other_end_time);
                let new_start_mins = time_to_minutes(&start_time);
                let new_end_mins = time_to_minutes(&end_time);
                let is_new_overnight = new_end_mins < new_start_mins;
                
                // New shift overlaps if it starts before the overnight shift ends
                // Or if the new shift is also overnight (they'd definitely overlap)
                new_start_mins < other_end_mins || is_new_overnight
            } else {
                // Same day - use normal overlap detection
                times_overlap(&start_time, &end_time, &other_start_time, &other_end_time)
            };
            
            debug_log!(state.debug, "[UPDATE ASSIGNED SHIFT] Overlap result: {}", has_overlap);
            
            if has_overlap {
                debug_log!(state.debug, "[UPDATE ASSIGNED SHIFT] ✗ Conflict: {} is already scheduled for {} shift in {} ({}-{}) on {}", 
                    emp_id, other_shift_typ, other_department_name, other_start_time, other_end_time, new_datum);
                return Err(ApiErrorResponse {
                    status: StatusCode::CONFLICT,
                    body: ApiError {
                        error: "scheduling_conflict".to_string(),
                        message: format!(
                            "Employee '{}' cannot be assigned to {} shift ({}-{}) in '{}' on {} because they are already scheduled for {} shift ({}-{}) in '{}'",
                            emp_name, shift_typ, start_time, end_time, department_name, new_datum,
                            other_shift_typ, other_start_time, other_end_time, other_department_name
                        ),
                        details: Some(ConflictDetails {
                            employee_id: emp_id.clone(),
                            employee_name: emp_name,
                            date: new_datum.clone(),
                            conflicting_shift: ConflictingShiftInfo {
                                shift_id: other_shift_id,
                                shift_type: other_shift_typ,
                                start_time: other_start_time,
                                end_time: other_end_time,
                                department_name: Some(other_department_name),
                            },
                            requested_shift: ShiftTimeInfo {
                                start_time: start_time.clone(),
                                end_time: end_time.clone(),
                            },
                        }),
                    },
                });
            }
        }
    }

    debug_log!(state.debug, "[UPDATE ASSIGNED SHIFT] ✓ No scheduling conflicts detected");

    let mut changes = Vec::new();
    if datum_changed {
        changes.push(format!("datum: {} → {}", current_datum, new_datum));
    }
    if mitarbeiter_changed {
        changes.push(format!("mitarbeiter_ids updated ({} employees)", new_mitarbeiter_ids.len()));
    }

    // Determine new state using helper function
    let new_state = determine_assigned_shift_state(
        new_mitarbeiter_ids.len(),
        template_arbeitsstunden,
        req.state,
        state.debug,
    );

    if new_state != current_state {
        changes.push(format!("state: {} → {}", current_state.as_str(), new_state.as_str()));
    }

    let new_mitarbeiter_json = serde_json::to_string(&new_mitarbeiter_ids)
        .unwrap_or_else(|_| "[]".to_string());

    sqlx::query("UPDATE assigned_shifts SET datum = ?, mitarbeiter_ids = ?, state = ? WHERE id = ?")
        .bind(&new_datum)
        .bind(&new_mitarbeiter_json)
        .bind(new_state.as_str())
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| ApiErrorResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: ApiError {
                error: "database_error".to_string(),
                message: format!("Failed to update assigned shift: {}", e),
                details: None,
            },
        })?;

    debug_log!(state.debug, "[UPDATE ASSIGNED SHIFT] ✓ Changes applied: {}", changes.join(", "));
    
    // Log updated employee assignments
    if !new_mitarbeiter_ids.is_empty() {
        let mut emp_names = Vec::new();
        for emp_id in &new_mitarbeiter_ids {
            let name = sqlx::query_scalar::<_, String>("SELECT name FROM employees WHERE id = ?")
                .bind(emp_id)
                .fetch_optional(&state.db)
                .await
                .ok()
                .flatten()
                .unwrap_or_else(|| emp_id.clone());
            emp_names.push(name);
        }
        debug_log!(state.debug, "[UPDATE ASSIGNED SHIFT] 📋 {} shift ({}-{}) in '{}' on {} now has {} employees: {}",
            shift_typ, start_time, end_time, department_name, new_datum,
            new_mitarbeiter_ids.len(), emp_names.join(", "));
    } else {
        debug_log!(state.debug, "[UPDATE ASSIGNED SHIFT] 📋 {} shift ({}-{}) in '{}' on {} is now unassigned",
            shift_typ, start_time, end_time, department_name, new_datum);
    }

    // Fetch created_at from database
    let created_at: Option<String> = sqlx::query_scalar(
        "SELECT strftime('%Y-%m-%dT%H:%M:%SZ', created_at) FROM assigned_shifts WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    Ok(Json(AssignedShift {
        id,
        datum: new_datum.clone(),
        arbeitsschichten_id: current_arbeitsschichten_id,
        mitarbeiter_ids: new_mitarbeiter_ids,
        state: new_state,
        start_utc: Some(compute_utc_timestamp(&new_datum, &start_time)),
        end_utc: Some(compute_utc_timestamp(&new_datum, &end_time)),
        created_at,
    }))
}

pub async fn delete_assigned_shift(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> StatusCode {
    debug_log!(state.debug, "[DELETE ASSIGNED SHIFT] Deleting assigned shift ID: {}", id);
    
    let result = sqlx::query("DELETE FROM assigned_shifts WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await;

    match result {
        Ok(result) if result.rows_affected() > 0 => {
            debug_log!(state.debug, "[DELETE ASSIGNED SHIFT] ✓ Assigned shift deleted");
            StatusCode::NO_CONTENT
        }
        Ok(_) => {
            debug_log!(state.debug, "[DELETE ASSIGNED SHIFT] ✗ Assigned shift not found");
            StatusCode::NOT_FOUND
        }
        Err(_) => {
            debug_log!(state.debug, "[DELETE ASSIGNED SHIFT] ✗ Error deleting assigned shift");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn get_department_assigned_shifts(
    State(state): State<AppState>,
    Path(department_id): Path<String>,
) -> Result<Json<Vec<AssignedShift>>, StatusCode> {
    debug_log!(state.debug, "[GET DEPARTMENT ASSIGNED SHIFTS] Fetching assigned shifts for department: {}", department_id);
    
    // Get assigned shifts for all templates in this department
    let rows = sqlx::query(
        "SELECT a.id, a.datum, a.arbeitsschichten_id, a.mitarbeiter_ids, a.state,
                s.start_time, s.end_time,
                strftime('%Y-%m-%dT%H:%M:%SZ', a.created_at) as created_at
         FROM assigned_shifts a
         JOIN shifts s ON a.arbeitsschichten_id = s.id
         WHERE s.department_id = ?"
    )
    .bind(&department_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let assigned_shifts: Vec<AssignedShift> = rows.into_iter().map(|row| {
        let mitarbeiter_json: String = row.get("mitarbeiter_ids");
        let mitarbeiter_ids: Vec<String> = serde_json::from_str(&mitarbeiter_json)
            .unwrap_or_default();
        let state_str: String = row.get("state");
        let shift_state = AssignedShiftState::from_str(&state_str).unwrap_or(AssignedShiftState::Valid);
        let datum: String = row.get("datum");
        let start_time: String = row.get("start_time");
        let end_time: String = row.get("end_time");
        
        AssignedShift {
            id: row.get("id"),
            datum: datum.clone(),
            arbeitsschichten_id: row.get("arbeitsschichten_id"),
            mitarbeiter_ids,
            state: shift_state,
            start_utc: Some(compute_utc_timestamp(&datum, &start_time)),
            end_utc: Some(compute_utc_timestamp(&datum, &end_time)),
            created_at: row.get("created_at"),
        }
    }).collect();

    debug_log!(state.debug, "[GET DEPARTMENT ASSIGNED SHIFTS] ✓ Found {} assigned shifts for department", assigned_shifts.len());
    Ok(Json(assigned_shifts))
}

pub async fn init_assigned_shifts(
    State(state): State<AppState>,
    Json(req): Json<InitAssignedShiftsRequest>,
) -> Result<Json<InitAssignedShiftsResponse>, StatusCode> {
    debug_log!(state.debug, "[INIT ASSIGNED SHIFTS] ========== START ==========");
    debug_log!(state.debug, "[INIT ASSIGNED SHIFTS] Request received - start_date: '{}', end_date: '{}', department_id: {:?}", 
        req.start_date, req.end_date, req.department_id);

    // Parse dates
    let start_date = match chrono::NaiveDate::parse_from_str(&req.start_date, "%Y-%m-%d") {
        Ok(date) => {
            debug_log!(state.debug, "[INIT ASSIGNED SHIFTS] ✓ Parsed start_date: {}", date);
            date
        }
        Err(e) => {
            debug_log!(state.debug, "[INIT ASSIGNED SHIFTS] ✗ Failed to parse start_date '{}': {}", req.start_date, e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let end_date = match chrono::NaiveDate::parse_from_str(&req.end_date, "%Y-%m-%d") {
        Ok(date) => {
            debug_log!(state.debug, "[INIT ASSIGNED SHIFTS] ✓ Parsed end_date: {}", date);
            date
        }
        Err(e) => {
            debug_log!(state.debug, "[INIT ASSIGNED SHIFTS] ✗ Failed to parse end_date '{}': {}", req.end_date, e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    if end_date < start_date {
        debug_log!(state.debug, "[INIT ASSIGNED SHIFTS] ✗ End date before start date: {} < {}", end_date, start_date);
        return Err(StatusCode::BAD_REQUEST);
    }

    debug_log!(state.debug, "[INIT ASSIGNED SHIFTS] Date range valid: {} to {}", start_date, end_date);

    // Get all shift templates (filter by department if provided)
    let shifts_query = if let Some(dept_id) = &req.department_id {
        debug_log!(state.debug, "[INIT ASSIGNED SHIFTS] Fetching shifts for department: {}", dept_id);
        match sqlx::query("SELECT id FROM shifts WHERE department_id = ?")
            .bind(dept_id)
            .fetch_all(&state.db)
            .await
        {
            Ok(result) => {
                debug_log!(state.debug, "[INIT ASSIGNED SHIFTS] ✓ Fetched {} shifts for department", result.len());
                result
            }
            Err(e) => {
                debug_log!(state.debug, "[INIT ASSIGNED SHIFTS] ✗ Database error fetching shifts: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    } else {
        debug_log!(state.debug, "[INIT ASSIGNED SHIFTS] Fetching all shifts (no department filter)");
        match sqlx::query("SELECT id FROM shifts").fetch_all(&state.db).await {
            Ok(result) => {
                debug_log!(state.debug, "[INIT ASSIGNED SHIFTS] ✓ Fetched {} shifts total", result.len());
                result
            }
            Err(e) => {
                debug_log!(state.debug, "[INIT ASSIGNED SHIFTS] ✗ Database error fetching shifts: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    };

    let shift_ids: Vec<String> = shifts_query.iter().map(|row| row.get("id")).collect();
    debug_log!(state.debug, "[INIT ASSIGNED SHIFTS] Extracted {} shift template IDs", shift_ids.len());

    if shift_ids.is_empty() {
        debug_log!(state.debug, "[INIT ASSIGNED SHIFTS] ⚠ No shift templates found! Check database or department_id");
    }

    let mut current_date = start_date;
    let mut created_count = 0;
    let mut error_count = 0;

    // For each date in range
    while current_date <= end_date {
        let datum = current_date.format("%Y-%m-%d").to_string();
        debug_log!(state.debug, "[INIT ASSIGNED SHIFTS] Processing date: {}", datum);

        // For each shift template
        for shift_id in &shift_ids {
            let new_id = uuid::Uuid::new_v4().to_string();
            let mitarbeiter_json = serde_json::to_string(&Vec::<String>::new())
                .unwrap_or_else(|_| "[]".to_string());

            debug_log!(state.debug, "[INIT ASSIGNED SHIFTS] Inserting: id={}, datum={}, shift_id={}", new_id, datum, shift_id);

            let result = sqlx::query(
                "INSERT INTO assigned_shifts (id, datum, arbeitsschichten_id, mitarbeiter_ids, state, created_at)
                 VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP)"
            )
            .bind(&new_id)
            .bind(&datum)
            .bind(shift_id)
            .bind(&mitarbeiter_json)
            .bind(AssignedShiftState::Unassigned.as_str())
            .execute(&state.db)
            .await;

            match result {
                Ok(_) => {
                    created_count += 1;
                    debug_log!(state.debug, "[INIT ASSIGNED SHIFTS] ✓ Inserted assigned shift");
                }
                Err(e) => {
                    error_count += 1;
                    debug_log!(state.debug, "[INIT ASSIGNED SHIFTS] ✗ Insert failed for shift {}: {}", shift_id, e);
                }
            }
        }

        current_date = current_date.succ_opt().unwrap_or(current_date);
    }

    let message = format!(
        "Created {} assigned shifts from {} to {} ({} errors)",
        created_count, req.start_date, req.end_date, error_count
    );

    debug_log!(state.debug, "[INIT ASSIGNED SHIFTS] ✓ {}", message);
    debug_log!(state.debug, "[INIT ASSIGNED SHIFTS] ========== END ==========");

    Ok(Json(InitAssignedShiftsResponse {
        created: created_count,
        start_date: req.start_date,
        end_date: req.end_date,
        message,
    }))
}

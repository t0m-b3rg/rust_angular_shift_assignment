use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// Enum for Schicht Types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SchichtType {
    #[serde(rename = "Früh")]
    Früh,
    #[serde(rename = "Mittel")]
    Mittel,
    #[serde(rename = "Spät")]
    Spät,
    #[serde(rename = "Nacht")]
    Nacht,
}

// Enum for AssignedShift State
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AssignedShiftState {
    Valid,
    Unassigned,
    #[serde(rename = "work_hours_not_satisfied")]
    WorkHoursNotSatisfied,
}

// Enum for Employee Qualifikation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Qualifikation {
    #[serde(rename = "Pflegehelfer")]
    Pflegehelfer,
    #[serde(rename = "Assistenzkraft")]
    Assistenzkraft,
    #[serde(rename = "Pflegefachkraft")]
    Pflegefachkraft,
}

impl Qualifikation {
    pub fn as_str(&self) -> &str {
        match self {
            Qualifikation::Pflegehelfer => "Pflegehelfer",
            Qualifikation::Assistenzkraft => "Assistenzkraft",
            Qualifikation::Pflegefachkraft => "Pflegefachkraft",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Pflegehelfer" => Some(Qualifikation::Pflegehelfer),
            "Assistenzkraft" => Some(Qualifikation::Assistenzkraft),
            "Pflegefachkraft" => Some(Qualifikation::Pflegefachkraft),
            _ => None,
        }
    }
}

impl AssignedShiftState {
    pub fn as_str(&self) -> &str {
        match self {
            AssignedShiftState::Valid => "valid",
            AssignedShiftState::Unassigned => "unassigned",
            AssignedShiftState::WorkHoursNotSatisfied => "work_hours_not_satisfied",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "valid" => Some(AssignedShiftState::Valid),
            "unassigned" => Some(AssignedShiftState::Unassigned),
            "work_hours_not_satisfied" => Some(AssignedShiftState::WorkHoursNotSatisfied),
            _ => None,
        }
    }
}

impl SchichtType {
    pub fn as_str(&self) -> &str {
        match self {
            SchichtType::Früh => "Früh",
            SchichtType::Mittel => "Mittel",
            SchichtType::Spät => "Spät",
            SchichtType::Nacht => "Nacht",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Früh" | "Frühschicht" => Some(SchichtType::Früh),
            "Mittel" | "Mittelschicht" => Some(SchichtType::Mittel),
            "Spät" | "Spätschicht" => Some(SchichtType::Spät),
            "Nacht" | "Nachtschicht" => Some(SchichtType::Nacht),
            _ => None,
        }
    }
}

// Data Models
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Department {
    pub id: String,
    pub name: String,
    pub bereich: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Employee {
    pub id: String,
    pub name: String,
    pub rolle: String,
    pub qualifikation: Qualifikation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Shift {
    pub id: String,
    pub datum: String,
    pub typ: String,  // Stored as normalized string: Früh, Mittel, Spät, Nacht
    pub start_time: String,
    pub end_time: String,
    pub department_id: String,
    pub arbeitsstunden: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

// Request Types
#[derive(Debug, Deserialize)]
pub struct CreateDepartmentRequest {
    pub name: String,
    pub bereich: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDepartmentRequest {
    pub name: Option<String>,
    pub bereich: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEmployeeRequest {
    pub name: String,
    pub rolle: String,
    pub qualifikation: Qualifikation,
}

#[derive(Debug, Deserialize)]
pub struct CreateShiftRequest {
    pub typ: SchichtType,
    pub start_time: String,
    pub end_time: String,
    pub department_id: String,
    pub arbeitsstunden: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEmployeeRequest {
    pub name: Option<String>,
    pub rolle: Option<String>,
    pub qualifikation: Option<Qualifikation>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateShiftRequest {
    pub typ: Option<SchichtType>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAssignedShiftRequest {
    pub datum: String,
    #[serde(rename = "arbeitsschichten_id")]
    pub arbeitsschichten_id: String,
    pub mitarbeiter_ids: Vec<String>,
    #[serde(default)]
    pub state: Option<AssignedShiftState>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAssignedShiftRequest {
    pub datum: Option<String>,
    pub mitarbeiter_ids: Option<Vec<String>>,
    pub state: Option<AssignedShiftState>,
}

#[derive(Debug, Deserialize)]
pub struct InitAssignedShiftsRequest {
    pub start_date: String,
    pub end_date: String,
    pub department_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct InitAssignedShiftsResponse {
    pub created: i32,
    pub start_date: String,
    pub end_date: String,
    pub message: String,
}

// Response type for assigned shifts - references a shift template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignedShift {
    pub id: String,
    pub datum: String,
    #[serde(rename = "arbeitsschichten_id")]
    pub arbeitsschichten_id: String,
    pub mitarbeiter_ids: Vec<String>,
    pub state: AssignedShiftState,
    /// UTC timestamp for shift start (datum + start_time)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_utc: Option<String>,
    /// UTC timestamp for shift end (datum + end_time)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_utc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

// Error Response Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<ConflictDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictDetails {
    pub employee_id: String,
    pub employee_name: String,
    pub date: String,
    pub conflicting_shift: ConflictingShiftInfo,
    pub requested_shift: ShiftTimeInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictingShiftInfo {
    pub shift_id: String,
    pub shift_type: String,
    pub start_time: String,
    pub end_time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub department_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShiftTimeInfo {
    pub start_time: String,
    pub end_time: String,
}

// JSON Import Types
#[derive(Debug, Deserialize)]
pub struct DemoDataJson {
    #[serde(rename = "pflegeheimName")]
    pub pflegeheim_name: String,
    pub datum: String,
    pub stationen: Vec<StationJson>,
    pub mitarbeiter: Vec<MitarbeiterJson>,
    #[serde(rename = "arbeitsschichten")]
    pub arbeitsschichten: Vec<ArbeitsschichtMusterJson>,
    #[serde(rename = "assigned_shifts")]
    pub assigned_shifts: Vec<AssignedShiftJson>,
}

#[derive(Debug, Deserialize)]
pub struct StationJson {
    pub id: String,
    pub name: String,
    pub bereich: String,
    pub pflegegrad: i32,
}

#[derive(Debug, Deserialize)]
pub struct MitarbeiterJson {
    pub id: String,
    pub name: String,
    pub rolle: String,
    pub qualifikation: Qualifikation,
}

#[derive(Debug, Deserialize)]
pub struct ArbeitsschichtMusterJson {
    pub id: String,
    pub typ: String,
    pub start: String,
    pub ende: String,
    pub station_id: String,
    pub arbeitsstunden: i32,
    pub arbeitszeit_faktor_nach_pflege_grad_und_schicht: i32,
}

#[derive(Debug, Deserialize)]
pub struct AssignedShiftJson {
    pub id: String,
    pub datum: String,
    #[serde(rename = "arbeitsschichten_id")]
    pub arbeitsschichten_id: String,
    pub mitarbeiter_ids: Vec<String>,
    #[serde(default)]
    pub state: Option<AssignedShiftState>,
}

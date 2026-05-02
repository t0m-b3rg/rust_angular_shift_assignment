# Architektur: Shift-Management System

## Übersicht

Das System zur Verwaltung von Arbeitsschichten folgt einer klaren Trennung zwischen **Shift-Templates** (wiederverwendbare Muster) und **Assigned Shifts** (spezifische Zuweisungen).

---

## 📊 Datenbankschema

```
┌─────────────────────────────────────────────────────────────┐
│                    DATENBANK SCHEMA                         │
└─────────────────────────────────────────────────────────────┘

┌──────────────────────┐
│   DEPARTMENTS        │
├──────────────────────┤
│ id (PK)              │
│ name                 │
│ bereich              │
│ created_at           │
└──────┬───────────────┘
       │
       │ 1:N
       │
       ├────────────────────────────────────────┐
       │                                        │
┌──────▼──────────────────┐        ┌───────────▼──────────┐
│     EMPLOYEES           │        │    SHIFTS            │
├─────────────────────────┤        ├──────────────────────┤
│ id (PK)                 │        │ id (PK)              │
│ name                    │        │ typ (Enum)           │
│ rolle                   │        │ start_time           │
│ qualifikation           │        │ end_time             │
│ department_id (FK)      │        │ department_id (FK)   │
│ created_at              │        │ arbeitsstunden       │
└─────────────────────────┘        │ created_at           │
                                   └──────┬───────────────┘
                                          │
                                          │ 1:N (Foreign Key)
                                          │
                                   ┌──────▼──────────────────────┐
                                   │   ASSIGNED_SHIFTS            │
                                   ├─────────────────────────────┤
                                   │ id (PK)                     │
                                   │ datum                       │
                                   │ arbeitsschichten_id (FK)    │
                                   │ mitarbeiter_ids (JSON)      │
                                   │ state (Enum)                │
                                   │ created_at                  │
                                   └─────────────────────────────┘
```

---

## 🔄 Konzeptionelle Architektur

### Alte Architektur (Problematisch)
```
┌─────────────────────────────────────────────────────────┐
│  SHIFTS (Monolithisch)                                  │
├─────────────────────────────────────────────────────────┤
│ • id, datum, typ, start_time, end_time                 │
│ • department_id, arbeitsstunden                        │
│ • Eindeutigkeit: (department_id, datum, typ)           │
│                                                         │
│ Problem: Redundanz bei wiederholten Schichten         │
│ Gleiche Früh-Schicht für jeden Tag duplizieren?        │
└─────────────────────────────────────────────────────────┘
```

### Neue Architektur (Optimiert)
```
┌──────────────────────────────────┐
│ SHIFTS (Template)                │
├──────────────────────────────────┤
│ • id: S01                        │
│ • typ: "Früh"                    │
│ • start_time: "06:00"            │
│ • end_time: "10:00"              │
│ • department_id: "ST01"          │
│ • arbeitsstunden: 8              │
│                                  │
│ Template: Wiederverwendbar!      │
└────────────┬─────────────────────┘
             │ Referenziert von (1:N)
             │
    ┌────────▼──────────────────────┐
    │ ASSIGNED_SHIFTS (Instanzen)    │
    ├────────────────────────────────┤
    │ • id: AS01                     │
    │ • datum: "2026-05-01"          │
    │ • arbeitsschichten_id: "S01"   │
    │ • mitarbeiter_ids: ["M01"]     │
    │                                │
    │ Instanz: Mit Datum & Personal! │
    └────────────────────────────────┘
```

---

## 📋 Daten-Flussdiagramm

### REST API Endpoints

```
SHIFT TEMPLATES (Arbeitsschichten)
═══════════════════════════════════════════════════════════
POST   /shifts
       → CreateShiftRequest {typ, start_time, end_time, ...}
       → Erstellt Template

GET    /shifts
       → Alle Templates auflisten

GET    /shifts/{id}
       → Spezifisches Template abrufen

PUT    /shifts/{id}
       → UpdateShiftRequest {typ?, start_time?, end_time?}
       → Template aktualisieren

DELETE /shifts/{id}
       → Template löschen (→ Löscht alle abhängigen Assignments!)

GET    /departments/{department_id}/shifts
       → Alle Templates einer Station


ASSIGNED SHIFTS (Instanzen)
═══════════════════════════════════════════════════════════
POST   /assigned-shifts
       → CreateAssignedShiftRequest {
           datum, arbeitsschichten_id, mitarbeiter_ids
         }
       → Erstellt Zuweisung für ein Datum
       → Konflikt-Prüfung: Überlappende Schichten werden abgelehnt

POST   /assigned-shifts/init
       → InitAssignedShiftsRequest {
           start_date, end_date, department_id?
         }
       → Bulk-Erstellung für Datumsbereich (alle Templates, leer)

GET    /assigned-shifts
       → Alle Zuweisungen auflisten (inkl. start_utc, end_utc)

GET    /assigned-shifts/{id}
       → Spezifische Zuweisung abrufen

PUT    /assigned-shifts/{id}
       → UpdateAssignedShiftRequest {datum?, mitarbeiter_ids?, state?}
       → Zuweisung aktualisieren
       → Konflikt-Prüfung bei Mitarbeiter-Änderungen

DELETE /assigned-shifts/{id}
       → Zuweisung löschen

GET    /departments/{department_id}/assigned-shifts
       → Alle Zuweisungen einer Station
```

---

## 🔐 EindeutigkeitsConstraints

### Shifts (Templates)
- **Keine Eindeutigkeitsbeschränkung auf Datenbankebene**
- Mehrere identische Templates sind möglich
- Schema: `(id)` ist der einzige Primary Key

### Assigned Shifts
- **Keine Eindeutigkeitsbeschränkung**
- Mehrere Zuweisungen können auf dasselbe Template für denselben Tag referenzieren
- Ermöglicht: Mehrere Mitarbeiter pro Schicht, Backups, etc.

---

## 📦 Datenmodelle (Rust)

### Request-Typen

```rust
// Shift Template erstellen
CreateShiftRequest {
    typ: SchichtType,           // Früh, Mittel, Spät, Nacht
    start_time: String,         // "06:00"
    end_time: String,           // "10:00"
    department_id: String,      // "ST01"
    arbeitsstunden: i32,        // 8
}

// Shift Template aktualisieren
UpdateShiftRequest {
    typ: Option<SchichtType>,
    start_time: Option<String>,
    end_time: Option<String>,
}

// Assigned Shift erstellen
CreateAssignedShiftRequest {
    datum: String,                      // "2026-05-01"
    arbeitsschichten_id: String,        // "S01" (Template-ID)
    mitarbeiter_ids: Vec<String>,       // ["M01", "M03"]
    state: Option<AssignedShiftState>,  // Optional: Override automatische State-Berechnung
}

// Assigned Shift aktualisieren
UpdateAssignedShiftRequest {
    datum: Option<String>,
    mitarbeiter_ids: Option<Vec<String>>,
    state: Option<AssignedShiftState>,  // Optional: Expliziter State-Override
}
```

### Response-Typen

```rust
// Shift Template (Response)
Shift {
    id: String,                 // "S01"
    typ: String,                // "Früh"
    start_time: String,         // "06:00"
    end_time: String,           // "10:00"
    department_id: String,      // "ST01"
    arbeitsstunden: i32,        // 8
    datum: String,              // Leer bei Templates
    created_at: Option<String>, // UTC timestamp
}

// Assigned Shift (Response)
AssignedShift {
    id: String,                         // "AS01"
    datum: String,                      // "2026-05-01"
    arbeitsschichten_id: String,        // "S01"
    mitarbeiter_ids: Vec<String>,       // ["M01", "M03"]
    state: AssignedShiftState,          // Valid | Unassigned | WorkHoursNotSatisfied
    start_utc: Option<String>,          // "2026-05-01T06:00:00Z" (computed)
    end_utc: Option<String>,            // "2026-05-01T10:00:00Z" (computed)
    created_at: Option<String>,         // UTC timestamp
}
```

---

## 🔄 AssignedShiftState Enum

Zustände für Schichtzuweisungen:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AssignedShiftState {
    Valid,                    // Ausreichend Mitarbeiter und Arbeitsstunden
    Unassigned,               // Keine Mitarbeiter zugewiesen
    WorkHoursNotSatisfied,    // Unzureichende Arbeitsstunden
}
```

### Automatische State-Bestimmung

Der State wird automatisch berechnet basierend auf:
- **Mitarbeiter-Anzahl**: 0 Mitarbeiter → `unassigned`
- **Arbeitsstunden**: `Mitarbeiter × 8 Stunden < benötigte Stunden` → `work_hours_not_satisfied`
- **Sonst**: `valid`

```rust
fn determine_assigned_shift_state(
    employee_count: usize,
    required_hours: i32,
    override_state: Option<AssignedShiftState>,
    debug: bool,
) -> AssignedShiftState
```

---

## 🎯 Anwendungsbeispiel

### Szenario: Tägliches Schichtwechsel-Management

#### Schritt 1: Templates definieren (einmalig)
```json
POST /shifts
{
  "typ": "Früh",
  "start_time": "06:00",
  "end_time": "10:00",
  "department_id": "ST01",
  "arbeitsstunden": 8
}
Response:
{
  "id": "S01",
  "typ": "Früh",
  "start_time": "06:00",
  "end_time": "10:00",
  "department_id": "ST01",
  "arbeitsstunden": 8,
  "datum": ""
}
```

#### Schritt 2: Für jeden Tag Zuweisungen erstellen
```json
POST /assigned-shifts
{
  "datum": "2026-05-01",
  "arbeitsschichten_id": "S01",
  "mitarbeiter_ids": ["M01", "M03"]
}
Response:
{
  "id": "AS001",
  "datum": "2026-05-01",
  "arbeitsschichten_id": "S01",
  "mitarbeiter_ids": ["M01", "M03"],
  "state": "valid",
  "start_utc": "2026-05-01T06:00:00Z",
  "end_utc": "2026-05-01T10:00:00Z"
}

POST /assigned-shifts
{
  "datum": "2026-05-02",
  "arbeitsschichten_id": "S01",
  "mitarbeiter_ids": ["M02", "M05"]
}
Response:
{
  "id": "AS002",
  "datum": "2026-05-02",
  "arbeitsschichten_id": "S01",
  "mitarbeiter_ids": ["M02", "M05"],
  "state": "valid",
  "start_utc": "2026-05-02T06:00:00Z",
  "end_utc": "2026-05-02T10:00:00Z"
}
```

#### Schritt 2b: Bulk-Initialisierung (Optional)
```json
POST /assigned-shifts/init
{
  "start_date": "2026-05-01",
  "end_date": "2026-05-07",
  "department_id": "ST01"
}
Response:
{
  "created": 28,
  "start_date": "2026-05-01",
  "end_date": "2026-05-07",
  "message": "Created 28 assigned shifts from 2026-05-01 to 2026-05-07"
}
```

#### Ergebnis: Template mit mehreren Instanzen
```
Template S01 (Früh-Schicht ST01, 06:00-10:00)
  │
  ├─ AS001 → 2026-05-01 [M01, M03]
  ├─ AS002 → 2026-05-02 [M02, M05]
  ├─ AS003 → 2026-05-03 [M04, M06]
  └─ AS004 → 2026-05-04 [M01, M02]
```

---

## 💾 Demodata-Struktur

```json
{
  "arbeitsschichten": [
    {
      "id": "S01",
      "typ": "Früh",
      "start": "06:00",
      "ende": "10:00",
      "station_id": "ST01",
      "arbeitsstunden": 8,
      "arbeitszeit_faktor_nach_pflege_grad_und_schicht": 4
    },
    ...12 Templates für 3 Stationen × 4 Schichttypen
  ],
  
  "assigned_shifts": [
    {
      "id": "AS01",
      "datum": "2026-05-01",
      "arbeitsschichten_id": "S01",
      "mitarbeiter_ids": [],
      "state": "unassigned"
    },
    ...12 Zuweisungen für den 1. Mai 2026
  ]
}
```

---

## 🔄 Datenbankoperationen

### Demo-Daten laden (init_demo_db.rs)

```
1. DELETE from assigned_shifts
   ↓
2. DELETE from shifts (Templates)
   ↓
3. DELETE from employees
   ↓
4. DELETE from departments
   ↓
5. INSERT INTO departments (FROM stationen)
   ↓
6. INSERT INTO employees (FROM mitarbeiter)
   ↓
7. INSERT INTO shifts (FROM arbeitsschichten) ← Templates!
   ↓
8. INSERT INTO assigned_shifts (FROM assigned_shifts) ← Instanzen!
   └─ mitarbeiter_ids als JSON-String serialisiert
```

---

## 🛡️ Fehlerbehandlung

### Error Response Typen

```rust
// Allgemeiner API-Fehler
ApiError {
    error: String,              // Fehler-Code (z.B. "conflict")
    message: String,            // Lesbare Beschreibung
    details: Option<ConflictDetails>,  // Optionale Details bei Konflikten
}

// Konflikt-Details
ConflictDetails {
    employee_id: String,        // "M01"
    employee_name: String,      // "Petra Müller"
    date: String,               // "2026-05-01"
    conflicting_shift: ConflictingShiftInfo,
    requested_shift: ShiftTimeInfo,
}
```

### Konflikt-Erkennung

Das System erkennt Scheduling-Konflikte bei:
- Erstellung von Assigned Shifts (`POST /assigned-shifts`)
- Aktualisierung von Mitarbeiter-Listen (`PUT /assigned-shifts/{id}`)

**Konflikt-Logik** (mit Nachtschicht-Unterstützung):
```rust
fn times_overlap(start1, end1, start2, end2) -> bool
// Handhabt normale Schichten UND Nachtschichten (22:00 → 06:00)
```

| Operation | Fehler | HTTP-Status | Grund |
|-----------|--------|------------|-------|
| POST /shifts | Department nicht gefunden | 400 Bad Request | department_id existiert nicht |
| POST /assigned-shifts | Template nicht gefunden | 400 Bad Request | arbeitsschichten_id existiert nicht |
| POST /assigned-shifts | Zeitkonflikt | 409 Conflict | Mitarbeiter hat überlappende Schicht |
| POST /assigned-shifts/init | Ungültiger Datumsbereich | 400 Bad Request | end_date < start_date |
| GET /shifts/{id} | Nicht gefunden | 404 Not Found | ID existiert nicht |
| PUT /assigned-shifts/{id} | Datum ungültig | 400 Bad Request | Format-Fehler |
| PUT /assigned-shifts/{id} | Zeitkonflikt | 409 Conflict | Mitarbeiter hat überlappende Schicht |
| DELETE /shifts/{id} | Template in Verwendung | 409 Conflict | Hätte abhängige Records |

---

## ✅ Vorteile der neuen Architektur

| Aspekt | Vorteil |
|--------|---------|
| **DRY Prinzip** | Templates reduzieren Datenredundanz |
| **Wartbarkeit** | Template-Änderung gilt für alle Zuweisungen (in Zukunft) |
| **Skalierbarkeit** | Ein Template → viele Zuweisungen über Monate/Jahre |
| **Flexibilität** | Verschiedene Mitarbeiter pro Datum/Template |
| **Nachverfolgung** | Jede Zuweisung hat eigene ID, Datum, Personal |
| **Normalisierung** | Datenbankschema folgt 3NF (Third Normal Form) |

---

## ⚙️ Helper-Funktionen (common.rs)

### Zeit-Verarbeitung

```rust
// Konvertiert HH:MM zu Minuten seit Mitternacht
fn time_to_minutes(time_str: &str) -> i32
// Beispiel: "06:00" → 360, "22:30" → 1350

// Prüft Überlappung zweier Zeiträume (inkl. Nachtschichten)
fn times_overlap(start1, end1, start2, end2) -> bool
// Handhabt: Normal (06:00-14:00) und Overnight (22:00-06:00)
```

### State-Bestimmung

```rust
// Automatische State-Berechnung für Assigned Shifts
fn determine_assigned_shift_state(
    employee_count: usize,
    required_hours: i32,
    override_state: Option<AssignedShiftState>,
    debug: bool,
) -> AssignedShiftState
```

### Debug Logging

```rust
// Makro für bedingte Debug-Ausgabe
debug_log!(state.debug, "[TAG] Message: {}", value);
// Aktiviert via DEBUG=true Umgebungsvariable
```

---

## 📝 Implementierungsdetails

### SchichtType Enum

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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

impl SchichtType {
    pub fn as_str(&self) -> &str { ... }
    pub fn from_str(s: &str) -> Option<Self> { ... }
}
```

### Qualifikation Enum

Qualifikationen für Pflegepersonal (Fachqualifikationen):

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Qualifikation {
    #[serde(rename = "Pflegefachkraft")]
    Pflegefachkraft,            // Professionelle Pflegefachkraft
    #[serde(rename = "Assistenzkraft")]
    Assistenzkraft,             // Pflegeassistenz/Spezialistin
    #[serde(rename = "Pflegehelfer")]
    Pflegehelfer,               // Pflegehelferin
}

impl Qualifikation {
    pub fn as_str(&self) -> &str { ... }
    pub fn from_str(s: &str) -> Option<Self> { ... }
}
```

### Employee Model

Mitarbeiter/Personalmodell mit Qualifikation (Enum):

| Feld | Typ | Beschreibung |
|------|-----|-------------|
| `id` | String | Eindeutige Mitarbeiter-ID (z.B. "M01") |
| `name` | String | Voller Name (z.B. "Petra Müller") |
| `rolle` | String | Job-Titel (z.B. "Pflegefachkraft") |
| `qualifikation` | Qualifikation (Enum) | Fachliche Qualifikation: Pflegefachkraft / Assistenzkraft / Pflegehelfer |

**Qualifikationshierarchie:**
- **Pflegefachkraft**: Höchste Qualifikation, professionelle Krankenpflegerin
- **Assistenzkraft**: Mittlere Qualifikation, spezialisierte Pflegeassistentin
- **Pflegehelfer**: Grundqualifikation, Pflegehelferin

### JSON-Serialisierung für mitarbeiter_ids

In der Datenbank werden `mitarbeiter_ids` als JSON-String gespeichert:
```sql
mitarbeiter_ids: '["M01","M03"]'  -- SQLite TEXT
```

In Rust werden sie deserialisiert:
```rust
let mitarbeiter_json: String = row.get("mitarbeiter_ids");
let mitarbeiter_ids: Vec<String> = serde_json::from_str(&mitarbeiter_json)
    .unwrap_or_default();  // [] bei Parse-Fehler
```

### UTC-Timestamp Berechnung

Die Felder `start_utc` und `end_utc` werden on-the-fly berechnet:

```rust
fn compute_utc_timestamp(datum: &str, time: &str) -> String {
    format!("{}T{}:00Z", datum, time)
}
// Beispiel: ("2026-05-01", "06:00") → "2026-05-01T06:00:00Z"
```

Diese Felder werden bei jedem Read-Vorgang berechnet und sind nicht in der DB gespeichert.

### InitAssignedShiftsRequest & InitAssignedShiftsResponse

Für Bulk-Erstellung von Schichtzuweisungen über einen Zeitraum:

```rust
#[derive(Debug, Deserialize)]
pub struct InitAssignedShiftsRequest {
    pub start_date: String,           // "YYYY-MM-DD" format
    pub end_date: String,             // "YYYY-MM-DD" format
    pub department_id: Option<String>, // Optional: filter by department
}

#[derive(Debug, Serialize)]
pub struct InitAssignedShiftsResponse {
    pub created: i32,                 // Number of created shifts
    pub start_date: String,
    pub end_date: String,
    pub message: String,              // Summary message
}
```

### init_assigned_shifts Handler Implementation

Handler-Funktion `pub async fn init_assigned_shifts()`:

```
1. Parse Request
   ├─ Parse start_date as chrono::NaiveDate (format: "%Y-%m-%d")
   ├─ Parse end_date as chrono::NaiveDate
   └─ Validate: end_date >= start_date (return 400 if invalid)

2. Query Shift Templates
   ├─ If department_id provided:
   │  └─ SELECT id FROM shifts WHERE department_id = ?
   └─ Else:
      └─ SELECT id FROM shifts

3. Iterate Over Date Range
   ├─ Start: start_date
   ├─ End: end_date (inclusive)
   └─ For each date:
      ├─ Format as "YYYY-MM-DD"
      └─ For each shift template:
         ├─ Generate new UUID for assignment
         ├─ Serialize empty Vec<String> to JSON: "[]"
         ├─ INSERT INTO assigned_shifts:
         │  ├─ id: UUID
         │  ├─ datum: current date
         │  ├─ arbeitsschichten_id: template id
         │  ├─ mitarbeiter_ids: "[]" (empty JSON)
         │  ├─ state: "unassigned"
         │  └─ created_at: CURRENT_TIMESTAMP
         └─ Increment created_count

4. Return Response
   ├─ created: total count
   ├─ start_date: request start_date
   ├─ end_date: request end_date
   └─ message: "Created X assigned shifts from DATE to DATE"
```

---

## 🚀 Zukunftserweiterungen

- [ ] Eindeutigkeitsconstraint auf `(arbeitsschichten_id, datum)` hinzufügen
- [ ] Template-Änderungen automatisch auf Zuweisungen propagieren
- [ ] Versionierung von Templates (Historie)
- [x] Bulk-Operations zum Erstellen von Zuweisungen über Zeiträume (init_assigned_shifts)
- [x] Konflikt-Erkennung (Mitarbeiter in mehreren Schichten) mit Nachtschicht-Unterstützung
- [x] AssignedShift State-System (Valid, Unassigned, WorkHoursNotSatisfied)
- [x] UTC-Timestamps für Schichtzeiten (start_utc, end_utc)
- [ ] Availability/Preference System

---

**Version:** 1.2 | **Datum:** 2026-05-02

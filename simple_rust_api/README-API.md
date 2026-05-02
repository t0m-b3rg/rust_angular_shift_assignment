# Shift-Management API

Eine REST-API zur Verwaltung von Arbeitsschichten, Mitarbeitern und Stationen in Pflegeeinrichtungen.

---

## 🛠️ Voraussetzungen

### Systemanforderungen

| Komponente | Version | Beschreibung |
|------------|---------|--------------|
| **Rust** | 1.70+ | Programmiersprache |
| **Cargo** | 1.70+ | Rust Paketmanager (wird mit Rust installiert) |

### Abhängigkeiten (werden automatisch installiert)

| Paket | Version | Beschreibung |
|-------|---------|--------------|
| **Tokio** | 1.40 | Asynchrone Runtime für Rust |
| **Axum** | 0.8.9 | Web-Framework für REST-APIs |
| **Serde** | 1.0 | JSON Serialisierung/Deserialisierung |
| **SQLx** | 0.7 | Asynchroner SQL-Datenbanktreiber |
| **tower-http** | 0.5 | HTTP-Middleware (CORS) |

---

## 🚀 Installation & Start

### Rust installieren

Falls Rust noch nicht installiert ist:

```bash
# Windows (PowerShell)
winget install Rustlang.Rust.MSVC

# Oder via rustup (alle Plattformen)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Projekt starten

```bash
# In das Projektverzeichnis wechseln
cd simple_rust_api

# Abhängigkeiten installieren und Server starten
cargo run

# Mit Debug-Ausgabe
DEBUG=1 cargo run

# Mit Demo-Daten
DEMO=1 cargo run

# Mit Debug UND Demo-Daten
DEBUG=1 DEMO=1 cargo run
```

### Server-Ausgabe

```
╔════════════════════════════════════════╗
║  🚀 Server running at http://127.0.0.1:3000  ║
║  Press Ctrl+C to stop                ║
╚════════════════════════════════════════╝
```

---

## 📦 Demo-Daten

Mit `DEMO=1` werden realistische Testdaten geladen:

### Mitarbeiter (9 Personen)

| ID | Name | Qualifikation |
|----|------|---------------|
| M01 | Petra Müller | Pflegefachkraft |
| M02 | Hans Schmidt | Pflegehelfer |
| M03 | Lisa Weber | Pflegefachkraft |
| M04 | Thomas Bauer | Pflegefachkraft |
| M05 | Anna Fischer | Pflegehelfer |
| M06 | Michael Hoffmann | Assistenzkraft |
| M07 | Silvia Rossi | Pflegefachkraft |
| M08 | Johannes Wagner | Pflegehelfer |
| M09 | Katrin Neumann | Assistenzkraft |

### Stationen (3 Bereiche)

| ID | Name | Bereich |
|----|------|---------|
| ST01 | Station A | Pflegebedürftige |
| ST02 | Station B | Demenz |
| ST03 | Station C | Normale Betreuung |

### Schicht-Templates (12 Vorlagen)

4 Templates pro Station:
- **Früh**: 06:00 - 10:00 (8 Stunden)
- **Mittel**: 10:00 - 16:00 (12 Stunden)
- **Spät**: 16:00 - 22:00 (8 Stunden)
- **Nacht**: 22:00 - 06:00 (16 Stunden)

### Zugewiesene Schichten (12 Einträge)

Eine Zuweisung pro Template für den 01.05.2026 (initial ohne Mitarbeiter).

---

## 📡 API-Endpoints

### Stationen (Departments)

| Methode | Endpoint | Beschreibung |
|---------|----------|--------------|
| `POST` | `/departments` | Neue Station erstellen |
| `GET` | `/departments` | Alle Stationen auflisten |
| `GET` | `/departments/{id}` | Station nach ID abrufen |
| `PUT` | `/departments/{id}` | Station aktualisieren |
| `DELETE` | `/departments/{id}` | Station löschen |

### Mitarbeiter (Employees)

| Methode | Endpoint | Beschreibung |
|---------|----------|--------------|
| `POST` | `/employees` | Neuen Mitarbeiter erstellen |
| `GET` | `/employees` | Alle Mitarbeiter auflisten |
| `GET` | `/employees/{id}` | Mitarbeiter nach ID abrufen |
| `PUT` | `/employees/{id}` | Mitarbeiter aktualisieren |
| `DELETE` | `/employees/{id}` | Mitarbeiter löschen |

### Schicht-Templates (Shifts)

| Methode | Endpoint | Beschreibung |
|---------|----------|--------------|
| `POST` | `/shifts` | Neues Schicht-Template erstellen |
| `GET` | `/shifts` | Alle Templates auflisten |
| `GET` | `/shifts/{id}` | Template nach ID abrufen |
| `PUT` | `/shifts/{id}` | Template aktualisieren |
| `DELETE` | `/shifts/{id}` | Template löschen |
| `GET` | `/departments/{id}/shifts` | Templates einer Station |

### Zugewiesene Schichten (Assigned Shifts)

| Methode | Endpoint | Beschreibung |
|---------|----------|--------------|
| `POST` | `/assigned-shifts` | Neue Zuweisung erstellen |
| `POST` | `/assigned-shifts/init` | Bulk-Initialisierung für Zeitraum |
| `GET` | `/assigned-shifts` | Alle Zuweisungen auflisten |
| `GET` | `/assigned-shifts/{id}` | Zuweisung nach ID abrufen |
| `PUT` | `/assigned-shifts/{id}` | Zuweisung aktualisieren |
| `DELETE` | `/assigned-shifts/{id}` | Zuweisung löschen |
| `GET` | `/departments/{id}/assigned-shifts` | Zuweisungen einer Station |

---

## 📋 Beispiel-Anfragen

### Station erstellen

```bash
curl -X POST http://127.0.0.1:3000/departments \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Station D",
    "bereich": "Intensivpflege"
  }'
```

### Mitarbeiter erstellen

```bash
curl -X POST http://127.0.0.1:3000/employees \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Maria Schmidt",
    "rolle": "Pflegekraft",
    "qualifikation": "Pflegefachkraft"
  }'
```

### Schicht-Template erstellen

```bash
curl -X POST http://127.0.0.1:3000/shifts \
  -H "Content-Type: application/json" \
  -d '{
    "typ": "Früh",
    "start_time": "06:00",
    "end_time": "14:00",
    "department_id": "ST01",
    "arbeitsstunden": 8
  }'
```

### Schicht zuweisen

```bash
curl -X POST http://127.0.0.1:3000/assigned-shifts \
  -H "Content-Type: application/json" \
  -d '{
    "datum": "2026-05-01",
    "arbeitsschichten_id": "S01",
    "mitarbeiter_ids": ["M01", "M03"]
  }'
```

### Bulk-Initialisierung (Wochenplan)

```bash
curl -X POST http://127.0.0.1:3000/assigned-shifts/init \
  -H "Content-Type: application/json" \
  -d '{
    "start_date": "2026-05-05",
    "end_date": "2026-05-11",
    "department_id": "ST01"
  }'
```

### Alle Daten abrufen

```bash
# Alle Mitarbeiter
curl http://127.0.0.1:3000/employees

# Alle Stationen
curl http://127.0.0.1:3000/departments

# Alle Schicht-Templates
curl http://127.0.0.1:3000/shifts

# Alle Zuweisungen
curl http://127.0.0.1:3000/assigned-shifts
```

---

## 💾 Datenbank

Die API verwendet **SQLite** zur persistenten Datenspeicherung. Die Datenbankdatei `simple_rust_api.db` wird automatisch beim Start erstellt.

### Datenbankschema

```sql
-- Stationen
CREATE TABLE departments (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    bereich TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Mitarbeiter
CREATE TABLE employees (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    rolle TEXT NOT NULL,
    qualifikation TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Schicht-Templates
CREATE TABLE shifts (
    id TEXT PRIMARY KEY,
    typ TEXT NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT NOT NULL,
    department_id TEXT NOT NULL,
    arbeitsstunden INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(department_id) REFERENCES departments(id) ON DELETE CASCADE
);

-- Zugewiesene Schichten
CREATE TABLE assigned_shifts (
    id TEXT PRIMARY KEY,
    datum TEXT NOT NULL,
    arbeitsschichten_id TEXT NOT NULL,
    mitarbeiter_ids TEXT NOT NULL,
    state TEXT DEFAULT 'unassigned',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(arbeitsschichten_id) REFERENCES shifts(id) ON DELETE CASCADE
);
```

### Datenbank direkt abfragen

```bash
# SQLite CLI öffnen
sqlite3 simple_rust_api.db

# Alle Stationen anzeigen
sqlite> SELECT * FROM departments;

# Alle Mitarbeiter anzeigen
sqlite> SELECT * FROM employees;

# Alle Schicht-Templates anzeigen
sqlite> SELECT * FROM shifts;

# Zuweisungen einer Station anzeigen
sqlite> SELECT * FROM assigned_shifts WHERE arbeitsschichten_id IN 
        (SELECT id FROM shifts WHERE department_id = 'ST01');
```

### Datenbank-Merkmale

- ✓ Persistente Speicherung über Server-Neustarts
- ✓ ACID-Transaktionen
- ✓ Foreign Key Constraints
- ✓ Kaskadierendes Löschen (Station löschen → Schichten & Zuweisungen löschen)
- ✓ Automatische Zeitstempel

---

## 🐛 Debug-Modus

Der Debug-Modus liefert detaillierte Informationen über alle Operationen.

### Aktivieren

```bash
DEBUG=1 cargo run
# oder
DEBUG=true cargo run
```

### Beispiel-Ausgabe

```
[DEBUG] Debug mode enabled

[CREATE ASSIGNED SHIFT] Received request - datum: 2026-05-01, arbeitsschichten_id: S01
[CREATE ASSIGNED SHIFT] ✓ Template verified: S01 (06:00 - 10:00) in Station A - 8 hours required
[CREATE ASSIGNED SHIFT] Checking conflicts for employee: M01
[CREATE ASSIGNED SHIFT] ✓ No scheduling conflicts detected
[CREATE ASSIGNED SHIFT] Generated assignment ID: abc123-...
[CREATE ASSIGNED SHIFT] ✓ Assigned shift created successfully
[CREATE ASSIGNED SHIFT] 📋 Assigned 2 employees to Früh shift (06:00-10:00) in 'Station A' on 2026-05-01: Petra Müller, Lisa Weber
```

### Debug-Ausgabe beinhaltet

- ✓ Eingehende Anfragen (Daten)
- ✓ Generierte IDs
- ✓ Validierungen (Template existiert, keine Konflikte)
- ✓ Erfolg/Fehler mit ✓/✗ Indikatoren
- ✓ Zugewiesene Mitarbeiter-Namen

---

## 🛡️ Fehlerbehandlung

### HTTP-Statuscodes

| Code | Bedeutung | Beispiel |
|------|-----------|----------|
| `200` | OK | Daten erfolgreich abgerufen |
| `201` | Created | Ressource erstellt |
| `204` | No Content | Ressource gelöscht |
| `400` | Bad Request | Ungültige Daten, fehlende Referenz |
| `404` | Not Found | Ressource existiert nicht |
| `409` | Conflict | Zeitkonflikt bei Mitarbeiter-Zuweisung |
| `500` | Server Error | Datenbankfehler |

### Fehler-Response (Beispiel: Konflikt)

```json
{
  "error": "conflict",
  "message": "Employee M01 (Petra Müller) already assigned to overlapping shift",
  "details": {
    "employee_id": "M01",
    "employee_name": "Petra Müller",
    "date": "2026-05-01",
    "conflicting_shift": {
      "shift_id": "AS001",
      "shift_type": "Früh",
      "start_time": "06:00",
      "end_time": "10:00",
      "department_name": "Station A"
    },
    "requested_shift": {
      "start_time": "08:00",
      "end_time": "14:00"
    }
  }
}
```

---

## 📊 Zuweisungs-Status (State)

Jede zugewiesene Schicht hat einen automatisch berechneten Status:

| Status | Beschreibung |
|--------|--------------|
| `valid` | Ausreichend Mitarbeiter und Arbeitsstunden |
| `unassigned` | Keine Mitarbeiter zugewiesen |
| `work_hours_not_satisfied` | Zu wenige Mitarbeiter für benötigte Stunden |

### Automatische Berechnung

```
Verfügbare Stunden = Anzahl Mitarbeiter × 8 Stunden

Wenn keine Mitarbeiter → "unassigned"
Wenn verfügbare < benötigte Stunden → "work_hours_not_satisfied"
Sonst → "valid"
```

---

## 🔗 Verwandte Dokumentation

- [ARCHITECTURE.md](ARCHITECTURE.md) - Detaillierte Systemarchitektur
- [WHAT_CAN_THIS_API_DO.md](WHAT_CAN_THIS_API_DO.md) - Funktionsübersicht

---

**Version:** 1.0 | **Stand:** 02.05.2026

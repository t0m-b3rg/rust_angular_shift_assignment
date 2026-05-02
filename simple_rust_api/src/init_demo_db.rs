use sqlx::SqlitePool;
use crate::models::DemoDataJson;

pub async fn init_demo_data(db: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    // Try multiple paths for the JSON file
    let possible_paths = vec![
        "src/demo_data.json",
        "demo_data.json",
        "../src/demo_data.json",
    ];

    let mut json_content = None;
    for path in &possible_paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            if !content.trim().is_empty() {
                json_content = Some(content);
                println!("[DEMO DATEN] ✓ demo_data.json gefunden unter: {}", path);
                break;
            }
        }
    }

    let json_content = match json_content {
        Some(content) => content,
        None => {
            println!("[DEMO DATEN] ⚠ demo_data.json nicht gefunden, demo wird übersprungen");
            return Ok(());
        }
    };

    let demo_data: DemoDataJson = serde_json::from_str(&json_content)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    // Clear existing data
    sqlx::query("DELETE FROM assigned_shifts").execute(db).await?;
    sqlx::query("DELETE FROM shifts").execute(db).await?;
    sqlx::query("DELETE FROM employees").execute(db).await?;
    sqlx::query("DELETE FROM departments").execute(db).await?;

    // Load departments (Stationen)
    for station in &demo_data.stationen {
        sqlx::query(
            "INSERT INTO departments (id, name, bereich) VALUES (?, ?, ?)"
        )
        .bind(&station.id)
        .bind(&station.name)
        .bind(&station.bereich)
        .execute(db)
        .await?;
    }

    // Load employees (Mitarbeiter)
    for mitarbeiter in &demo_data.mitarbeiter {
        sqlx::query(
            "INSERT INTO employees (id, name, rolle, qualifikation) VALUES (?, ?, ?, ?)"
        )
        .bind(&mitarbeiter.id)
        .bind(&mitarbeiter.name)
        .bind(&mitarbeiter.rolle)
        .bind(mitarbeiter.qualifikation.as_str())
        .execute(db)
        .await?;
    }

    // Load shift templates (Arbeitsschichten)
    for schicht in &demo_data.arbeitsschichten {
        sqlx::query(
            "INSERT INTO shifts (id, typ, start_time, end_time, department_id, arbeitsstunden) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&schicht.id)
        .bind(&schicht.typ)
        .bind(&schicht.start)
        .bind(&schicht.ende)
        .bind(&schicht.station_id)
        .bind(schicht.arbeitsstunden)
        .execute(db)
        .await?;
    }

    // Load assigned shifts (Assigned Shifts)
    for assigned in &demo_data.assigned_shifts {
        let mitarbeiter_json = serde_json::to_string(&assigned.mitarbeiter_ids)
            .unwrap_or_else(|_| "[]".to_string());
        let state = assigned.state.as_ref().map(|s| s.as_str()).unwrap_or("valid");
        
        sqlx::query(
            "INSERT INTO assigned_shifts (id, datum, arbeitsschichten_id, mitarbeiter_ids, state) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&assigned.id)
        .bind(&assigned.datum)
        .bind(&assigned.arbeitsschichten_id)
        .bind(&mitarbeiter_json)
        .bind(state)
        .execute(db)
        .await?;
    }

    // Print summary
    println!("\n╔════════════════════════════════════════════════════════════════════╗");
    println!("║  [DEMO DATEN] ✓ Demo-Daten erfolgreich geladen!                   ║");
    println!("╠════════════════════════════════════════════════════════════════════╣");
    println!("║  Pflegeheim: {}                              ║", demo_data.pflegeheim_name);
    println!("║  Datum: {}                                                   ║", demo_data.datum);
    println!("║                                                                    ║");
    println!("║  Stationen: {}                                                   ║", demo_data.stationen.len());
    for station in &demo_data.stationen {
        println!("║    • {} ({})", station.name, station.bereich);
    }
    println!("║                                                                    ║");
    println!("║  Mitarbeiter: {}                                                 ║", demo_data.mitarbeiter.len());
    for mitarbeiter in &demo_data.mitarbeiter {
        println!("║    • {} - {} ({})", mitarbeiter.name, mitarbeiter.rolle, mitarbeiter.qualifikation.as_str());
    }
    println!("║                                                                    ║");
    println!("║  Shift-Templates: {}                                              ║", demo_data.arbeitsschichten.len());
    println!("║  Assigned Shifts: {}                                              ║", demo_data.assigned_shifts.len());
    println!("╚════════════════════════════════════════════════════════════════════╝\n");

    Ok(())
}

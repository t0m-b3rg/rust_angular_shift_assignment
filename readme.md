# Schichtverwaltungssystem für Pflegeeinrichtungen

Ein Prototyp für ein Schichtverwaltungssystem im medizinischen Kontext, am Beispiel eines Pflegeheims entwickelt.

---

## 📋 Projektübersicht

Dieses Projekt besteht aus zwei Komponenten:

| Komponente | Technologie | Verzeichnis | Dokumentation |
|------------|-------------|-------------|---------------|
| **Frontend** | Angular 16+ | `fe/` | [README-FE.md](fe/README-FE.md) |
| **Backend** | Rust (Axum) | `simple_rust_api/` | [README-API.md](simple_rust_api/README-API.md) |

### Frontend (Angular)

Das Frontend bietet eine moderne Benutzeroberfläche mit Angular Material für die Verwaltung von Schichten, Mitarbeitern und Stationen. Weitere Informationen zur Installation und den verfügbaren Funktionen finden sich in der [Frontend-Dokumentation](fe/README-FE.md).

### Backend (Rust)

Das Backend stellt eine REST-API auf Basis von Axum bereit, die alle notwendigen Endpunkte für die Schichtverwaltung zur Verfügung stellt. Details zur API und den Endpunkten sind in der [Backend-Dokumentation](simple_rust_api/README-API.md) beschrieben.

---

## 🎯 Schwerpunkt der Implementierung

Der Fokus dieser Implementierung liegt auf:

- **Zuweisung von Mitarbeitern zu Schichten** – Die zentrale Funktionalität des Systems
- **Erkennung und Handhabung von Schichtüberlappungen** – Vermeidung von Konflikten bei der Dienstplanung

---

## ⚠️ Hinweise zum Entwicklungsstand

### Einschränkungen

Einige ursprünglich geplante Funktionen konnten im Rahmen dieses Prototyps nicht vollständig umgesetzt werden:

- Abstufungen der Arbeitszeit nach Pflegegrad
- Differenzierung der Arbeitsleistung nach Qualifikation
- Vollständige CRUD-Operationen über die Benutzeroberfläche

### Datenverwaltung

Mitarbeiter, Schichten und Stationen (Abteilungen) können aktuell nur über die Datei `demo_data.json` geändert werden. Die entsprechenden API-Endpunkte zum Hinzufügen und Modifizieren sind implementiert, jedoch fehlt die vollständige Integration in das Frontend.

---

## 🚀 Schnellstart

1. **Backend starten:**
   ```bash
   cd simple_rust_api
   DEMO=1 cargo run
   ```

2. **Frontend starten:**
   ```bash
   cd fe
   npm install
   npm start
   ```

3. **Anwendung öffnen:** [http://localhost:4200](http://localhost:4200)

---

## 📝 Fazit

Dieses Projekt war ambitioniert angelegt – die Überreste im Code zeugen von den ursprünglich geplanten, aber nicht umgesetzten Features. Dennoch bildet der Prototyp die Kernfunktionalität eines Schichtverwaltungssystems ab und demonstriert das Zusammenspiel von Angular-Frontend und Rust-Backend.

Viel Spaß beim Ausprobieren! Mir hat die Entwicklung Freude bereitet.

---

*Grüße, Thomas*

# Schicht-Management System - Angular Frontend

Ein Angular 16+ Frontend für die Pflegeheim Schichtverwaltungs-REST-API (Rust/Axum Backend).

---

## 🛠️ Voraussetzungen

### Systemanforderungen

| Komponente | Version | Beschreibung |
|------------|---------|--------------|
| **Node.js** | 16+ | JavaScript Runtime |
| **npm** | 8+ | Node Paketmanager (wird mit Node.js installiert) |
| **Angular CLI** | 16+ | Angular Kommandozeilen-Tool |

### Backend-Anforderung

Das Rust-Backend muss unter `http://127.0.0.1:3000` laufen. Siehe [README-API.md](../simple_rust_api/README-API.md) für Backend-Installation.

---

## 🚀 Installation & Start

### Node.js installieren

Falls Node.js noch nicht installiert ist:

```bash
# Windows (PowerShell)
winget install OpenJS.NodeJS.LTS

# Oder von https://nodejs.org herunterladen
```

### Angular CLI installieren

```bash
npm install -g @angular/cli
```

### Projekt starten

```bash
# In das Projektverzeichnis wechseln
cd fe

# Abhängigkeiten installieren
npm install

# Entwicklungsserver starten
npm start

# Oder mit automatischem Browser-Öffnen
npm run dev
```

### Verfügbare Befehle

| Befehl | Beschreibung |
|--------|--------------|
| `npm start` | Entwicklungsserver starten (http://localhost:4200) |
| `npm run dev` | Entwicklungsserver mit Browser-Öffnen |
| `npm run build` | Entwicklungs-Build erstellen |
| `npm run build:prod` | Produktions-Build erstellen |
| `npm test` | Unit-Tests ausführen |
| `npm run lint` | Linter ausführen |

Die Anwendung ist unter **http://localhost:4200** erreichbar.

---

## ✨ Funktionen

- **Mitarbeiterverwaltung**: Erstellen, Aktualisieren und Verwalten von Personal mit Qualifikationen
- **Stationsverwaltung**: Organisation von Stationen für das Pflegeheim
- **Schicht-Templates**: Definition wiederverwendbarer Schichtmuster (Früh, Mittel, Spät, Nacht)
- **Dienstplanverwaltung**: Mitarbeiter zu Schichten zuweisen mit Konflikt-Erkennung
- **Bulk-Operationen**: Initialisierung von Schichtzuweisungen für Zeiträume
- **Material Design**: Moderne Benutzeroberfläche mit Angular Material
- **Type-Safe**: Vollständige TypeScript-Unterstützung mit Strict-Mode

---

## 📁 Projektstruktur

```
src/
├── app/
│   ├── modules/
│   │   ├── dashboard/        # Dashboard-Übersicht
│   │   ├── employees/        # Mitarbeiterverwaltung
│   │   ├── departments/      # Stationsverwaltung
│   │   ├── shifts/           # Schicht-Template-Verwaltung
│   │   └── schedule/         # Dienstplan und Zuweisungen
│   ├── services/             # HTTP-Services
│   ├── models/               # TypeScript-Interfaces
│   ├── shared/               # Gemeinsame Komponenten/Pipes
│   ├── app.module.ts         # Root-Modul
│   └── app.component.ts      # Root-Komponente
├── environments/             # Umgebungskonfigurationen
├── styles.scss               # Globale Styles
└── theme.scss                # Material-Theme
```

---

## 🔗 API-Integration

Das Frontend kommuniziert mit dem Rust-Backend unter `http://127.0.0.1:3000`. Die Umgebungskonfiguration befindet sich in `src/environments/environment.ts`.

### Verfügbare Endpoints

| Ressource | Methoden | Endpoint |
|-----------|----------|----------|
| Mitarbeiter | GET/POST/PUT/DELETE | `/employees` |
| Stationen | GET/POST/PUT/DELETE | `/departments` |
| Schicht-Templates | GET/POST/PUT/DELETE | `/shifts` |
| Zugewiesene Schichten | GET/POST/PUT/DELETE | `/assigned-shifts` |
| Bulk-Initialisierung | POST | `/assigned-shifts/init` |

---

## 📋 Geschäftsregeln

| Regel | Beschreibung |
|-------|--------------|
| **Konflikt-Erkennung** | System verhindert Zuweisung von Mitarbeitern zu überlappenden Schichten |
| **Arbeitsstunden-Validierung** | Jeder Mitarbeiter zählt als 8 Arbeitsstunden pro Tag |
| **Qualifikationen** | Pflegefachkraft, Assistenzkraft, Pflegehelfer |
| **Status-Verwaltung** | Schichten werden auf Arbeitsstunden-Abdeckung validiert |

---

## 🧰 Entwicklung

### Neue Komponente erstellen

```bash
ng generate component modules/ihr-modul/components/ihre-komponente
```

### Neuen Service erstellen

```bash
ng generate service services/ihr-service
```

### Tests ausführen

```bash
# Unit-Tests
npm test

# Mit Coverage-Report
ng test --code-coverage
```

### Build für Produktion

```bash
npm run build:prod
```

Die Build-Ausgabe befindet sich im `dist/` Verzeichnis.

---

## 📦 Abhängigkeiten

### Produktionsabhängigkeiten

| Paket | Version | Beschreibung |
|-------|---------|--------------|
| @angular/core | ^16.0.0 | Angular Framework |
| @angular/material | ^16.0.0 | Material Design Komponenten |
| @angular/cdk | ^16.0.0 | Component Dev Kit |
| @angular/router | ^16.0.0 | Angular Router |
| rxjs | ^7.8.0 | Reactive Extensions |

### Entwicklungsabhängigkeiten

| Paket | Version | Beschreibung |
|-------|---------|--------------|
| @angular/cli | ^16.0.0 | Angular CLI |
| typescript | ~5.1.0 | TypeScript Compiler |
| karma | ~6.4.0 | Test Runner |
| jasmine-core | ~4.6.0 | Test Framework |

---

## 🐛 Fehlerbehebung

### Port bereits belegt

```bash
# Anderen Port verwenden
ng serve --port 4201
```

### Backend nicht erreichbar

Stellen Sie sicher, dass das Rust-Backend läuft:
```bash
cd ../simple_rust_api
cargo run
```

### Node-Module neu installieren

```bash
rm -rf node_modules
npm install
```

---

## 🔗 Verwandte Dokumentation

- [README-API.md](../simple_rust_api/README-API.md) - Backend API Dokumentation
- [ARCHITECTURE.md](../simple_rust_api/ARCHITECTURE.md) - Systemarchitektur

---

**Version:** 1.0 | **Stand:** 02.05.2026

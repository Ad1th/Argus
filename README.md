# AetherQuery

AetherQuery is a local full-stack app that turns CSV files into queryable DuckDB views, then helps you run SQL and visualize execution plans in an interactive React UI.

## One-Line GitHub Description

Upload CSVs, run SQL on in-memory DuckDB, and visualize query execution plans through a FastAPI + React interface.

## What It Does

AetherQuery lets you:

- Upload a CSV file and register it as a DuckDB view.
- Inspect schema and preview data.
- Execute ad-hoc SQL queries against uploaded data.
- Compute simple per-column stats.
- Parse and visualize SQL execution plans as an operator graph.

## Project Structure

- `backend/` - FastAPI server, DuckDB connection, and API routers.
- `frontend/` - Vite + React TypeScript UI for upload, querying, and plan visualization.
- `employees.csv` - Sample dataset for quick testing.

## Tech Stack

Backend:

- Python 3.11+
- FastAPI
- DuckDB (in-memory)
- Uvicorn

Frontend:

- React 19 + TypeScript
- Vite
- React Flow (plan graph rendering)
- Dagre (graph layout dependency)

## Architecture Overview

1. CSV is uploaded to the backend.
2. Backend writes a temp file under `/tmp` and creates a DuckDB view using `read_csv_auto`.
3. Frontend stores the returned `table_name` and offers suggested SQL queries.
4. Query execution and plan parsing endpoints are called from the frontend.
5. Parsed plan tree is rendered as a node-edge graph in React Flow.

Note: DuckDB is currently in-memory (`:memory:`), so uploaded tables reset when the backend restarts.

## Getting Started

### Prerequisites

- Python 3.11+
- Node.js 18+ and npm

### 1. Clone and enter the project

```bash
git clone <your-repo-url>
cd AetherQuery
```

### 2. Start the backend

```bash
python3 -m venv venv
source venv/bin/activate
pip install fastapi uvicorn duckdb==0.9.2 python-multipart
uvicorn backend.main:app --reload
```

Backend runs at:

- `http://127.0.0.1:8000`
- Swagger docs: `http://127.0.0.1:8000/docs`

### 3. Start the frontend

Open a second terminal:

```bash
cd frontend
npm install
npm run dev
```

Frontend runs at:

- `http://localhost:5173`

## How To Use

1. Open the frontend in your browser.
2. Upload a CSV file.
3. Pick a suggested query or write your own SQL.
4. Click Analyze Query to generate and visualize the plan tree.
5. Click Run Query to view tabular results.

## API Reference

Base URL: `http://127.0.0.1:8000`

### Health

- `GET /`
  - Returns backend status message.

### Data Ingestion

- `POST /api/upload`
  - Form-data: `file` (CSV)
  - Returns: generated `table_name` and temp `path`

### Schema + Data Inspection

- `GET /api/schema/{table_name}`
  - Returns column names and types.
- `GET /api/preview/{table_name}?limit=50`
  - Returns columns and preview rows.
- `GET /api/stats/{table_name}`
  - Returns per-column min, max, avg, distinct count, null count.

### SQL Execution

- `POST /api/sql/execute`
  - JSON body:

```json
{
  "query": "SELECT * FROM my_table LIMIT 10"
}
```

- Returns `success`, `columns`, and `rows` (or `error`).

### Query Plan Parsing

- `POST /api/sql/parse-plan`
  - JSON body:

```json
{
  "query": "SELECT AVG(salary) FROM my_table"
}
```

- Returns `plan_tree` plus a short natural-language explanation.

### Analyze Endpoint

- `POST /api/sql/analyze`
  - Runs `EXPLAIN ANALYZE` and returns textual plan output.

## Current Limitations

- SQL strings are executed directly; there is no sandboxing or access control.
- Query plan parsing currently supports a limited set of operators and uses heuristic parsing.
- DuckDB is process-local in-memory storage; data is not persisted.
- CORS is open (`allow_origins=["*"]`) for local development convenience.

## Troubleshooting

### Backend fails to start

- Ensure your virtual environment is active.
- Reinstall dependencies:

```bash
pip install --upgrade fastapi uvicorn duckdb==0.9.2 python-multipart
```

### Frontend cannot connect to backend

- Confirm backend is running on `http://127.0.0.1:8000`.
- Confirm frontend is running on `http://localhost:5173`.
- If needed, check browser console for failed network requests.

### Upload works but query fails

- Verify table name returned by upload and use that exact name.
- Confirm column names in your SQL match the uploaded schema.

## Development Notes

- Backend entrypoint: `backend/main.py`
- DuckDB connection: `backend/duckdb_engine.py`
- Plan parser route: `backend/routers/plan.py`
- SQL execution route: `backend/routers/sql.py`
- Main frontend page: `frontend/src/App.tsx`
- Plan graph component: `frontend/src/components/PlanGraph.tsx`

## Future Improvements

- Persist DuckDB state to a file-backed database.
- Harden SQL execution with validation and table scoping.
- Expand operator parsing to support joins, filters, and nested plans.
- Improve graph layout for deterministic node placement.
- Add tests for backend routes and plan parser logic.

## License

Add your preferred license (for example, MIT) in a `LICENSE` file.

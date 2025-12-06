from fastapi import APIRouter
from backend.duckdb_engine import get_connection

router = APIRouter()

@router.get("/preview/{table_name}")
async def preview_table(table_name: str, limit: int = 50):
    conn = get_connection()
    rows = conn.execute(f"SELECT * FROM {table_name} LIMIT {limit}").fetchall()
    columns = [desc[0] for desc in conn.description]

    return {
        "columns": columns,
        "rows": rows
    } 
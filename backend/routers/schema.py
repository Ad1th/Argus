from fastapi import APIRouter
from backend.duckdb_engine import get_connection

router = APIRouter()

@router.get("/schema/{table_name}")
async def get_schema(table_name: str):
    conn = get_connection()
    result = conn.execute(f"DESCRIBE {table_name}").fetchall()
    schema = [
        {
            "column": row[0],
            "type": row[1]
        }
        for row in result
    ]
    return {"schema" : schema}
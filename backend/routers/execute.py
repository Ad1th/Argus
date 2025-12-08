from fastapi import APIRouter
from pydantic import BaseModel
from backend.duckdb_engine import get_connection

router = APIRouter()

class QueryRequest(BaseModel):
    query: str

@router.post("/execute")
async def execute_query(req: QueryRequest):
    conn = get_connection()
    try:
        result = conn.execute(req.query).fetchall()
        columns = [c[0] for c in conn.description]
        return {
            "success": True,
            "columns": columns,
            "rows": result
        }
    except Exception as e:
        return {
            "success": False,
            "error": str(e)
        }
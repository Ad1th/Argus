from fastapi import APIRouter
from pydantic import BaseModel
from backend.duckdb_engine import get_connection
import time

router = APIRouter()

class QueryRequest(BaseModel):
    query: str
    table: str | None = None
    
@router.post("/execute")
async def execute_query(req: QueryRequest):
    conn = get_connection()

    start = time.time()

    try:
        result = conn.execute(req.query)
        rows = result.fetchall()
        columns = [desc[0] for desc in result.description]

        end = time.time()

        return {
            "success": True,
            "columns": columns,
            "rows": rows,
            "execution_time_ms": round((end - start) * 1000, 3)
        }

    except Exception as e:
        return {
            "success": False,
            "error": str(e)
        }
    
# @router.post("/analyze")
# async def analyze_query(req: QueryRequest):
#     conn = get_connection()
#     try:
#         plan = conn.execute(f"EXPLAIN (FORMAT TEXT) {req.query}").fetchall()
#         explain_text = "\n".join(str(row[0]) for row in plan)
#         return {
#             "success": True,
#             "plan": explain_text
#         }
#     except Exception as e:
#         return {
#             "success": False,
#             "error": str(e)
#         }
    

# @router.post("/analyze")
# async def analyze_query(req: QueryRequest):
#     conn = get_connection()

#     try:
#         # Request JSON output (works on all modern DuckDB versions)
#         rows = conn.execute(
#             f"EXPLAIN (FORMAT JSON) {req.query}"
#         ).fetchall()

#         # DuckDB returns one row with one JSON string
#         plan_json = rows[0][0]

#         return {
#             "success": True,
#             "plan": plan_json
#         }

#     except Exception as e:
#         return {
#             "success": False,
#             "error": str(e)
#         }

@router.post("/analyze")
async def analyze_query(req: QueryRequest):
    conn = get_connection()

    try:
        rows = conn.execute(f"EXPLAIN {req.query}").fetchall()

        # DuckDB 0.9.2 returns: (plan_type, plan_text)
        plan_text = rows[0][1]

        return {
            "success": True,
            "plan": plan_text
        }
    except Exception as e:
        return {
            "success": False,
            "error": str(e)
        }
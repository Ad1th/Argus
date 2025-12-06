from fastapi import APIRouter
from backend.duckdb_engine import get_connection

router = APIRouter()

@router.get("/stats/{table_name}")
async def table_stats(table_name: str):
    conn = get_connection()
    cols = conn.execute(f"DESCRIBE {table_name}").fetchall()
    col_names = [c[0] for c in cols]
    stats = {}
    for col in col_names:
        query = f"""
            SELECT 
                MIN({col}), 
                MAX({col}),
                AVG({col}),
                COUNT(DISTINCT {col}),
                SUM(CASE WHEN {col} IS NULL THEN 1 ELSE 0 END)
            FROM {table_name};
        """
        result = conn.execute(query).fetchone()
        stats[col] = {
            "min": result[0],
            "max": result[1],
            "avg": result[2],
            "distinct": result[3],
            "nulls": result[4]
        }

    return stats

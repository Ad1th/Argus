from fastapi import FastAPI
from backend.routers import upload, schema, preview, stats, sql, plan

app = FastAPI()

app.include_router(upload.router, prefix="/api")
app.include_router(schema.router, prefix = "/api")
app.include_router(preview.router, prefix="/api")
app.include_router(stats.router, prefix="/api")
app.include_router(sql.router, prefix="/api/sql")
app.include_router(plan.router, prefix="/api/sql")

@app.get("/")
def root():
    return{"msg": "AetherQuery backend is running"}



"""
/opt/homebrew/bin/python3.11 -m venv venv
source venv/bin/activate
python3 --version
pip install duckdb==0.9.2

python3 - << 'EOF'
import duckdb
con = duckdb.connect()
print(con.execute("EXPLAIN SELECT 1").fetchall())
EOF

"""

"""
cd ~/Documents/Dev/Projects/AetherQuery
source venv/bin/activate
uvicorn backend.main:app --reload
"""

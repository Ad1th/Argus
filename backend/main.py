from fastapi import FastAPI
from backend.routers import upload, schema, preview, stats, sql, plan
from fastapi.middleware.cors import CORSMiddleware

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



app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_methods=["*"],
    allow_headers=["*"],
)



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


table_ec879a2297fe4504bdc4869239471662

import duckdb
con = duckdb.connect()

print(con.execute("EXPLAIN SELECT AVG(salary) FROM table_ec879a2297fe4504bdc4869239471662").fetchall())
"""

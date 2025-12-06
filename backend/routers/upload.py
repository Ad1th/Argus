from fastapi import APIRouter, UploadFile, File
import shutil, uuid
from backend.duckdb_engine import get_connection

router =APIRouter()

@router.post("/upload")
async def upload_csv(file: UploadFile = File(...)):
    file_id = str(uuid.uuid4()) #this is to save the file temporarily
    temp_path = f"/tmp/{file_id}.csv"
    with open(temp_path, "wb") as buffer:
        shutil.copyfileobj(file.file, buffer)

    #this block to create the table in duckdb
    conn = get_connection() 
    table_name = f"table_{file_id.replace('-', '')}"
    conn.execute(f"""
    CREATE VIEW {table_name} AS SELECT * FROM read_csv_auto('{temp_path}');
    """)
    return {
        "table_name": table_name,
        "path": temp_path
    }
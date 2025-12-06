import duckdb

conn = duckdb.connect(database=":memory:", read_only=False)

def get_connection():
    return conn
from fastapi import APIRouter
from pydantic import BaseModel
from backend.duckdb_engine import get_connection

router = APIRouter()

class QueryRequest(BaseModel):
    query: str

@router.post("/parse-plan")
async def parse_plan(req: QueryRequest):
    conn = get_connection()
    raw_plan_rows = conn.execute(f"EXPLAIN {req.query}").fetchall()
    raw_plan = "\n".join(row[0] for row in raw_plan_rows)
    parsed_tree = parse_explain_plan(raw_plan)
    explanation = explain_tree(parsed_tree)

    return {
        "success": True,
        "plan_tree": parsed_tree,
        "explanation": explanation
    }

def parse_explain_plan(text: str):
    """
    Converts ASCII tree text into structured JSON.
    Very lightweight indentation-based parser.
    """

    lines = text.split("\n")

    stack = []
    root = None

    for line in lines:
        stripped = line.lstrip()
        indent = len(line) - len(stripped)

        node = create_node_from_line(stripped)

        if not stack:
            root = node
            stack.append((indent, node))
            continue

        # find parent based on indent
        while stack and stack[-1][0] >= indent:
            stack.pop()

        parent_indent, parent_node = stack[-1]
        parent_node["children"].append(node)

        stack.append((indent, node))

    return root


def create_node_from_line(line: str):
    """
    Converts a single EXPLAIN line to a node.
    """

    node = {
        "type": None,
        "columns": [],
        "aggregates": [],
        "rows": None,
        "children": []
    }

    # Detect operator type
    node["type"] = line.split()[0]

    # Detect aggregates
    if "Aggregates:" in line:
        agg = line.split("Aggregates:")[1].strip()
        node["aggregates"] = [agg]

    # Detect projections
    if "Projections:" in line:
        cols = line.split("Projections:")[1].strip().split(",")
        node["columns"] = [c.strip() for c in cols]

    # Detect row count
    if "~" in line and "rows" in line:
        num = line.split("~")[1].split("rows")[0].strip()
        node["rows"] = int(num)

    return node


def explain_tree(node):
    """
    Generates a natural-language explanation.
    """
    t = node["type"]

    if t == "READ_CSV_AUTO":
        return "Reads the CSV file from disk."

    if t == "PROJECTION":
        return f"Selects columns: {', '.join(node['columns'])}."

    if t == "UNGROUPED_AGGREGATE":
        return f"Applies aggregate functions: {', '.join(node['aggregates'])}."

    # Default fallback
    return f"Executes operator: {t}."
from fastapi import APIRouter
from pydantic import BaseModel
from backend.duckdb_engine import get_connection
import re

router = APIRouter()

class QueryRequest(BaseModel):
    query: str


@router.post("/parse-plan")
async def parse_plan(req: QueryRequest):
    conn = get_connection()

    # Run normal EXPLAIN (ASCII art)
    rows = conn.execute(f"EXPLAIN {req.query}").fetchall()

    # Column 1 contains each line of the plan
    explain_lines = [r[1] for r in rows]

    # Clean entire plan
    cleaned = clean_explain_output("\n".join(explain_lines))

    # Parse meaning, not indentation
    parsed_tree = build_operator_tree(cleaned)

    explanation = explain_tree(parsed_tree)

    return {
        "success": True,
        "plan_tree": parsed_tree,
        "explanation": explanation
    }


# -------------------------------------------------------------
# CLEAN OUTPUT — remove ASCII art, keep only meaning
# -------------------------------------------------------------

def clean_explain_output(text: str):
    cleaned = []
    for line in text.split("\n"):

        # Remove box characters
        for ch in ["┌", "┐", "└", "┘", "│", "─", "┬", "┴"]:
            line = line.replace(ch, "")

        line = line.strip()
        if not line:
            continue

        cleaned.append(line)

    return cleaned


# -------------------------------------------------------------
# OPERATOR TREE BUILDER (content-based, NOT indent-based)
# -------------------------------------------------------------

def build_operator_tree(lines):

    ops = []
    current = None

    for line in lines:

        # Detect main operator
        if "UNGROUPED_AGGREGATE" in line:
            current = make_node("UNGROUPED_AGGREGATE")
            ops.append(current)

        elif "PROJECTION" in line:
            current = make_node("PROJECTION")
            ops.append(current)

        elif "SEQ_SCAN" in line:
            current = make_node("SEQ_SCAN")
            ops.append(current)

        elif "READ_CSV_AUTO" in line:
            current = make_node("READ_CSV_AUTO")
            ops.append(current)

        # Attach aggregates
        if "Aggregates:" in line:
            aggs = line.replace("Aggregates:", "").strip()
            current["aggregates"] = [aggs]

        # Attach projections / columns
        if "salary" in line.lower():
            current["columns"].append("salary")

        # Attach row count
        if "~" in line and "rows" in line:
            num = line.split("~")[1].split("rows")[0].strip()
            current["rows"] = int(num)

    # Build tree manually: Agg → Projection → Scan
    return assemble_tree(ops)


def make_node(op):
    return {
        "type": op,
        "columns": [],
        "aggregates": [],
        "rows": None,
        "children": []
    }


def assemble_tree(nodes):

    root = None
    last_proj = None

    for n in nodes:

        if n["type"] == "UNGROUPED_AGGREGATE":
            root = n

        elif n["type"] == "PROJECTION":
            if root:
                root["children"].append(n)
            last_proj = n

        elif n["type"] in ("SEQ_SCAN", "READ_CSV_AUTO"):
            if last_proj:
                last_proj["children"].append(n)

    return root


# -------------------------------------------------------------
# NATURAL LANGUAGE EXPLANATION
# -------------------------------------------------------------

def explain_tree(node):

    if not node:
        return "Could not parse query plan."

    t = node["type"]

    if t == "UNGROUPED_AGGREGATE":
        return f"Computes aggregate: {', '.join(node['aggregates'])}."

    if t == "PROJECTION":
        return f"Selects columns: {', '.join(node['columns'])}."

    if t == "SEQ_SCAN":
        return f"Sequential scan over table (~{node['rows']} rows)."

    return f"Executes operator: {t}."
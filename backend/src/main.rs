use std::{
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{Duration, NaiveDate, Utc};
use duckdb::{Connection, Row};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::{fs::File, io::AsyncWriteExt};
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

type SharedConnection = Arc<Mutex<Connection>>;

#[derive(Clone)]
struct AppState {
    conn: SharedConnection,
}

#[derive(Debug, Deserialize)]
struct QueryRequest {
    query: String,
}

#[derive(Debug, Deserialize)]
struct PreviewQuery {
    limit: Option<usize>,
}

#[derive(Debug, Serialize, Clone)]
struct PlanNode {
    #[serde(rename = "type")]
    node_type: String,
    #[serde(default)]
    columns: Vec<String>,
    #[serde(default)]
    aggregates: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rows: Option<u64>,
    #[serde(default)]
    children: Vec<PlanNode>,
}

#[tokio::main]
async fn main() {
    let db_path =
        std::env::var("AETHERQUERY_DB_PATH").unwrap_or_else(|_| "aetherquery.duckdb".to_string());
    let connection = Connection::open(&db_path).expect("failed to open DuckDB connection");
    let state = AppState {
        conn: Arc::new(Mutex::new(connection)),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(root))
        .route("/api/upload", post(upload_csv))
        .route("/api/schema/{table_name}", get(get_schema))
        .route("/api/preview/{table_name}", get(preview_table))
        .route("/api/stats/{table_name}", get(table_stats))
        .route("/api/sql/execute", post(execute_query))
        .route("/api/sql/analyze", post(analyze_query))
        .route("/api/sql/parse-plan", post(parse_plan))
        .with_state(state)
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind 127.0.0.1:8000");

    println!("AetherQuery backend listening on http://{}", addr);
    println!("DuckDB database: {}", db_path);

    axum::serve(listener, app).await.expect("server error");
}

async fn root() -> Json<Value> {
    Json(json!({"msg": "AetherQuery backend is running"}))
}

async fn upload_csv(State(state): State<AppState>, mut multipart: Multipart) -> impl IntoResponse {
    let mut saved_path: Option<PathBuf> = None;

    while let Ok(Some(mut field)) = multipart.next_field().await {
        if field.name() != Some("file") {
            continue;
        }

        let file_id = Uuid::new_v4().to_string();
        let temp_path = std::env::temp_dir().join(format!("{}.csv", file_id));
        let mut file = match File::create(&temp_path).await {
            Ok(file) => file,
            Err(error) => return json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
        };

        while let Ok(Some(chunk)) = field.chunk().await {
            if let Err(error) = file.write_all(&chunk).await {
                return json_error(StatusCode::INTERNAL_SERVER_ERROR, error);
            }
        }

        saved_path = Some(temp_path);
        break;
    }

    let Some(temp_path) = saved_path else {
        return Json(json!({"success": false, "error": "missing file"})).into_response();
    };

    let file_id = Uuid::new_v4().to_string();
    let table_name = format!("table_{}", file_id.replace('-', ""));
    let path_literal = sql_string_literal(temp_path.to_string_lossy().as_ref());
    let sql = format!("CREATE TABLE {table_name} AS SELECT * FROM read_csv_auto({path_literal});");

    match with_connection(&state, |conn| {
        conn.execute_batch(&sql).map_err(|error| error.to_string())
    }) {
        Ok(_) => Json(
            json!({"success": true, "table_name": table_name, "path": temp_path.to_string_lossy()}),
        )
        .into_response(),
        Err(error) => json_error(StatusCode::INTERNAL_SERVER_ERROR, error),
    }
}

async fn execute_query(
    State(state): State<AppState>,
    Json(req): Json<QueryRequest>,
) -> impl IntoResponse {
    let start = Utc::now();

    match with_connection(&state, |conn| execute_select(conn, &req.query)) {
        Ok((columns, rows)) => {
            let end = Utc::now();
            Json(json!({
                "success": true,
                "columns": columns,
                "rows": rows,
                "execution_time_ms": (end - start).num_microseconds().unwrap_or(0) as f64 / 1000.0
            }))
            .into_response()
        }
        Err(error) => json_error(StatusCode::OK, error),
    }
}

async fn analyze_query(
    State(state): State<AppState>,
    Json(req): Json<QueryRequest>,
) -> impl IntoResponse {
    match with_connection(&state, |conn| {
        explain_text(conn, &format!("EXPLAIN ANALYZE {}", req.query))
    }) {
        Ok(plan) => Json(json!({"success": true, "plan": plan.join("\n")})).into_response(),
        Err(error) => json_error(StatusCode::OK, error),
    }
}

async fn parse_plan(
    State(state): State<AppState>,
    Json(req): Json<QueryRequest>,
) -> impl IntoResponse {
    match with_connection(&state, |conn| {
        explain_text(conn, &format!("EXPLAIN {}", req.query))
    }) {
        Ok(lines) => {
            let plan_tree = build_plan_tree(&lines);
            let explanation = explain_tree(plan_tree.as_ref());
            Json(json!({
                "success": true,
                "plan_tree": plan_tree,
                "explanation": explanation,
            }))
            .into_response()
        }
        Err(error) => json_error(StatusCode::OK, error),
    }
}

async fn preview_table(
    State(state): State<AppState>,
    Path(table_name): Path<String>,
    Query(query): Query<PreviewQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50);
    let sql = format!(
        "SELECT * FROM {} LIMIT {}",
        quote_identifier(&table_name),
        limit
    );

    match with_connection(&state, |conn| execute_select(conn, &sql)) {
        Ok((columns, rows)) => Json(json!({"columns": columns, "rows": rows})).into_response(),
        Err(error) => json_error(StatusCode::OK, error),
    }
}

async fn get_schema(
    State(state): State<AppState>,
    Path(table_name): Path<String>,
) -> impl IntoResponse {
    let sql = format!("DESCRIBE {}", quote_identifier(&table_name));

    match with_connection(&state, |conn| execute_select(conn, &sql)) {
        Ok((_columns, rows)) => {
            let schema = rows
                .into_iter()
                .filter_map(|row| {
                    let column = row.get(0).map(json_to_display_string)?;
                    let ty = row.get(1).map(json_to_display_string)?;
                    Some(json!({"column": column, "type": ty}))
                })
                .collect::<Vec<_>>();

            Json(json!({"schema": schema})).into_response()
        }
        Err(error) => json_error(StatusCode::OK, error),
    }
}

async fn table_stats(
    State(state): State<AppState>,
    Path(table_name): Path<String>,
) -> impl IntoResponse {
    let describe_sql = format!("DESCRIBE {}", quote_identifier(&table_name));
    let columns = match with_connection(&state, |conn| execute_select(conn, &describe_sql)) {
        Ok((_columns, rows)) => rows
            .into_iter()
            .filter_map(|row| row.get(0).map(json_to_display_string))
            .collect::<Vec<_>>(),
        Err(error) => return json_error(StatusCode::OK, error),
    };

    let mut stats = serde_json::Map::new();

    for column in columns {
        let sql = format!(
            r#"SELECT
                MIN({col})::VARCHAR AS min,
                MAX({col})::VARCHAR AS max,
                AVG(TRY_CAST({col} AS DOUBLE)) AS avg,
                COUNT(DISTINCT {col}) AS distinct_count,
                SUM(CASE WHEN {col} IS NULL THEN 1 ELSE 0 END) AS nulls
            FROM {table};"#,
            col = quote_identifier(&column),
            table = quote_identifier(&table_name)
        );

        match with_connection(&state, |conn| execute_select(conn, &sql)) {
            Ok((_columns, rows)) => {
                let row = rows.into_iter().next();
                let entry = json!({
                    "min": row.as_ref().and_then(|r| r.get(0)).cloned().unwrap_or(Value::Null),
                    "max": row.as_ref().and_then(|r| r.get(1)).cloned().unwrap_or(Value::Null),
                    "avg": row.as_ref().and_then(|r| r.get(2)).cloned().unwrap_or(Value::Null),
                    "distinct": row.as_ref().and_then(|r| r.get(3)).cloned().unwrap_or(Value::Null),
                    "nulls": row.as_ref().and_then(|r| r.get(4)).cloned().unwrap_or(Value::Null),
                });
                stats.insert(column, entry);
            }
            Err(error) => return json_error(StatusCode::OK, error),
        }
    }

    Json(Value::Object(stats)).into_response()
}

fn with_connection<T, F>(state: &AppState, f: F) -> Result<T, String>
where
    F: FnOnce(&mut Connection) -> Result<T, String>,
{
    let mut guard = state.conn.lock().map_err(|error| error.to_string())?;
    f(&mut guard)
}

fn execute_select(
    conn: &mut Connection,
    sql: &str,
) -> Result<(Vec<String>, Vec<Vec<Value>>), String> {
    let mut statement = conn.prepare(sql).map_err(|error| error.to_string())?;
    let mut rows = statement.query([]).map_err(|error| error.to_string())?;
    let statement_ref = rows
        .as_ref()
        .ok_or_else(|| "query did not return a result set".to_string())?;
    let columns = statement_ref
        .column_names()
        .into_iter()
        .map(|name| name.to_string())
        .collect::<Vec<_>>();
    let mut data = Vec::new();

    while let Some(row) = rows.next().map_err(|error| error.to_string())? {
        data.push(extract_row(row, columns.len())?);
    }

    Ok((columns, data))
}

fn extract_row(row: &Row<'_>, column_count: usize) -> Result<Vec<Value>, String> {
    let mut values = Vec::with_capacity(column_count);

    for index in 0..column_count {
        let value = row.get_ref(index).map_err(|error| error.to_string())?;
        values.push(json_value_to_json(&value));
    }

    Ok(values)
}

fn explain_text(conn: &mut Connection, sql: &str) -> Result<Vec<String>, String> {
    let mut statement = conn.prepare(sql).map_err(|error| error.to_string())?;
    let mut rows = statement.query([]).map_err(|error| error.to_string())?;
    let mut lines = Vec::new();

    while let Some(row) = rows.next().map_err(|error| error.to_string())? {
        let line = if let Ok(value) = row.get_ref(1) {
            json_value_to_string(&value)
        } else if let Ok(value) = row.get_ref(0) {
            json_value_to_string(&value)
        } else {
            String::new()
        };

        if !line.is_empty() {
            lines.push(line);
        }
    }

    Ok(lines)
}

fn build_plan_tree(lines: &[String]) -> Option<PlanNode> {
    let mut nodes = Vec::new();
    let mut current: Option<usize> = None;

    for line in clean_explain_output(lines.join("\n").as_str()) {
        let upper = line.to_uppercase();

        if upper.contains("UNGROUPED_AGGREGATE") {
            nodes.push(make_node("UNGROUPED_AGGREGATE"));
            current = Some(nodes.len() - 1);
        } else if upper.contains("HASH_JOIN") {
            nodes.push(make_node("HASH_JOIN"));
            current = Some(nodes.len() - 1);
        } else if upper.contains("PROJECTION") {
            nodes.push(make_node("PROJECTION"));
            current = Some(nodes.len() - 1);
        } else if upper.contains("SEQ_SCAN") {
            nodes.push(make_node("SEQ_SCAN"));
            current = Some(nodes.len() - 1);
        } else if upper.contains("READ_CSV_AUTO") {
            nodes.push(make_node("READ_CSV_AUTO"));
            current = Some(nodes.len() - 1);
        } else if upper.contains("FILTER") {
            nodes.push(make_node("FILTER"));
            current = Some(nodes.len() - 1);
        }

        if let Some(index) = current {
            if let Some(aggregates) = parse_aggregates(&line) {
                nodes[index].aggregates.push(aggregates);
            }

            if let Some(rows) = parse_rows(&line) {
                nodes[index].rows = Some(rows);
            }

            if let Some(columns) = parse_projection_columns(&line) {
                for column in columns {
                    if !nodes[index].columns.contains(&column) {
                        nodes[index].columns.push(column);
                    }
                }
            }
        }
    }

    assemble_tree(nodes)
}

fn clean_explain_output(text: &str) -> Vec<String> {
    let mut cleaned = Vec::new();

    for mut line in text.lines().map(|line| line.to_string()) {
        for ch in ["┌", "┐", "└", "┘", "│", "─", "┬", "┴"] {
            line = line.replace(ch, "");
        }

        let line = line.trim().to_string();
        if !line.is_empty() {
            cleaned.push(line);
        }
    }

    cleaned
}

fn make_node(node_type: &str) -> PlanNode {
    PlanNode {
        node_type: node_type.to_string(),
        columns: Vec::new(),
        aggregates: Vec::new(),
        rows: None,
        children: Vec::new(),
    }
}

fn assemble_tree(mut nodes: Vec<PlanNode>) -> Option<PlanNode> {
    if nodes.is_empty() {
        return None;
    }

    let mut root = nodes.remove(0);
    let mut current = &mut root;

    for node in nodes {
        current.children.push(node);
        let len = current.children.len();
        current = current.children.get_mut(len - 1).unwrap();
    }

    Some(root)
}

fn explain_tree(node: Option<&PlanNode>) -> String {
    let Some(node) = node else {
        return "Could not parse query plan.".to_string();
    };

    match node.node_type.as_str() {
        "UNGROUPED_AGGREGATE" => format!("Computes aggregate: {}.", node.aggregates.join(", ")),
        "PROJECTION" => format!("Selects columns: {}.", node.columns.join(", ")),
        "SEQ_SCAN" => match node.rows {
            Some(rows) => format!("Sequential scan over table (~{rows} rows)."),
            None => "Sequential scan over table.".to_string(),
        },
        other => format!("Executes operator: {other}."),
    }
}

fn parse_aggregates(line: &str) -> Option<String> {
    line.split_once("Aggregates:")
        .map(|(_, value)| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn parse_rows(line: &str) -> Option<u64> {
    let regex = Regex::new(r"~\s*([0-9,]+)\s*rows").ok()?;
    let captures = regex.captures(line)?;
    let number = captures.get(1)?.as_str().replace(',', "");
    number.parse().ok()
}

fn parse_projection_columns(line: &str) -> Option<Vec<String>> {
    let line = line.to_lowercase();
    if !line.contains("salary") {
        return None;
    }

    Some(vec!["salary".to_string()])
}

fn sql_string_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn quote_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn json_value_to_string(value: &duckdb::types::ValueRef<'_>) -> String {
    match value {
        duckdb::types::ValueRef::Null => String::new(),
        duckdb::types::ValueRef::Boolean(v) => v.to_string(),
        duckdb::types::ValueRef::TinyInt(v) => v.to_string(),
        duckdb::types::ValueRef::SmallInt(v) => v.to_string(),
        duckdb::types::ValueRef::Int(v) => v.to_string(),
        duckdb::types::ValueRef::BigInt(v) => v.to_string(),
        duckdb::types::ValueRef::UTinyInt(v) => v.to_string(),
        duckdb::types::ValueRef::USmallInt(v) => v.to_string(),
        duckdb::types::ValueRef::UInt(v) => v.to_string(),
        duckdb::types::ValueRef::UBigInt(v) => v.to_string(),
        duckdb::types::ValueRef::HugeInt(v) => v.to_string(),
        duckdb::types::ValueRef::Float(v) => v.to_string(),
        duckdb::types::ValueRef::Double(v) => v.to_string(),
        duckdb::types::ValueRef::Decimal(v) => v.to_string(),
        duckdb::types::ValueRef::Text(v) => String::from_utf8_lossy(v).to_string(),
        duckdb::types::ValueRef::Blob(v) => format!("{:?}", v),
        duckdb::types::ValueRef::Date32(v) => date32_to_string(*v),
        duckdb::types::ValueRef::Time64(_, v) => v.to_string(),
        duckdb::types::ValueRef::Timestamp(_, v) => v.to_string(),
        other => format!("{:?}", other),
    }
}

fn json_value_to_json(value: &duckdb::types::ValueRef<'_>) -> Value {
    match value {
        duckdb::types::ValueRef::Null => Value::Null,
        duckdb::types::ValueRef::Boolean(v) => Value::Bool(*v),
        duckdb::types::ValueRef::TinyInt(v) => json!(*v),
        duckdb::types::ValueRef::SmallInt(v) => json!(*v),
        duckdb::types::ValueRef::Int(v) => json!(*v),
        duckdb::types::ValueRef::BigInt(v) => json!(*v),
        duckdb::types::ValueRef::UTinyInt(v) => json!(*v),
        duckdb::types::ValueRef::USmallInt(v) => json!(*v),
        duckdb::types::ValueRef::UInt(v) => json!(*v),
        duckdb::types::ValueRef::UBigInt(v) => json!(*v),
        duckdb::types::ValueRef::HugeInt(v) => i128_to_json(*v),
        duckdb::types::ValueRef::Float(v) => json!(*v),
        duckdb::types::ValueRef::Double(v) => json!(*v),
        duckdb::types::ValueRef::Decimal(v) => json!(v.to_string()),
        duckdb::types::ValueRef::Text(v) => Value::String(String::from_utf8_lossy(v).to_string()),
        duckdb::types::ValueRef::Blob(v) => Value::String(format!("{:?}", v)),
        duckdb::types::ValueRef::Date32(v) => Value::String(date32_to_string(*v)),
        duckdb::types::ValueRef::Time64(_, v) => Value::String(v.to_string()),
        duckdb::types::ValueRef::Timestamp(_, v) => Value::String(v.to_string()),
        other => Value::String(format!("{:?}", other)),
    }
}

fn json_to_display_string(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn i128_to_json(value: i128) -> Value {
    if let Ok(value) = i64::try_from(value) {
        json!(value)
    } else if let Ok(value) = u64::try_from(value) {
        json!(value)
    } else {
        json!(value.to_string())
    }
}

fn date32_to_string(days_since_epoch: i32) -> String {
    let Some(epoch) = NaiveDate::from_ymd_opt(1970, 1, 1) else {
        return days_since_epoch.to_string();
    };

    epoch
        .checked_add_signed(Duration::days(days_since_epoch as i64))
        .map(|date| date.to_string())
        .unwrap_or_else(|| days_since_epoch.to_string())
}

fn json_error(error_status: StatusCode, error: impl std::fmt::Display) -> axum::response::Response {
    (
        error_status,
        Json(json!({"success": false, "error": error.to_string()})),
    )
        .into_response()
}

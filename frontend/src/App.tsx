import React, { useState } from "react";
import PlanGraph from "./components/PlanGraph";

function App() {
  const [tableName, setTableName] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const [plan, setPlan] = useState<any | null>(null);
  const [result, setResult] = useState<any | null>(null);
  const [suggestedQueries, setSuggestedQueries] = useState<string[]>([]);

  const backend = "http://127.0.0.1:8000";

  // -----------------------
  // Upload CSV Handler
  // -----------------------
  const handleUpload = async (e: any) => {
    const file = e.target.files[0];
    if (!file) return;

    const form = new FormData();
    form.append("file", file);

    const res = await fetch(`${backend}/api/upload`, {
      method: "POST",
      body: form,
    });

    const data = await res.json();

    if (!data.table_name) {
      alert("Upload failed");
      return;
    }

    const tbl = data.table_name;
    setTableName(tbl);

    setSuggestedQueries([
      `SELECT * FROM ${tbl} LIMIT 5;`,
      `SELECT COUNT(*) FROM ${tbl};`,
      `SELECT AVG(salary) FROM ${tbl};`,
    ]);
  };

  // -----------------------
  // Analyze SQL Plan
  // -----------------------
  const handleAnalyze = async () => {
    if (!query.trim()) return;

    const res = await fetch(`${backend}/api/sql/parse-plan`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ query }),
    });

    const data = await res.json();
    setPlan(data.plan_tree ?? null);
  };

  // -----------------------
  // Execute SQL Query
  // -----------------------
  const handleExecute = async () => {
    if (!query.trim()) return;

    const res = await fetch(`${backend}/api/sql/execute`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ query }),
    });

    const data = await res.json();
    setResult(data);
  };

  return (
    <div
      style={{
        padding: "20px",
        color: "white",
        background: "#1e1e1e",
        minHeight: "100vh",
      }}
    >
      <h1>AetherQuery — Plan Visualizer</h1>

      {/* ---------------- Upload CSV ---------------- */}
      <div style={{ marginTop: "20px", marginBottom: "20px" }}>
        <label style={{ fontSize: "16px", fontWeight: "bold" }}>
          Upload CSV File:
        </label>
        <br />
        <input
          type="file"
          onChange={handleUpload}
          style={{
            marginTop: "10px",
            padding: "5px",
            border: "1px solid #555",
            background: "#333",
            color: "white",
          }}
        />
      </div>

      {tableName && (
        <p>
          <strong>Loaded Table:</strong> {tableName}
        </p>
      )}

      {/* ---------------- Suggested Queries ---------------- */}
      {tableName && (
        <div style={{ marginTop: "20px" }}>
          <h3>Suggested Queries:</h3>

          {suggestedQueries.map((q, i) => (
            <div
              key={i}
              style={{
                marginBottom: "8px",
                display: "flex",
                alignItems: "center",
                gap: "10px",
              }}
            >
              <code>{q}</code>
              <button
                onClick={() => setQuery(q)}
                style={{
                  padding: "4px 10px",
                  background: "#444",
                  color: "white",
                  borderRadius: "4px",
                }}
              >
                Use Query
              </button>
            </div>
          ))}
        </div>
      )}

      {/* ---------------- Query Textbox ---------------- */}
      <textarea
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        placeholder="Enter SQL query..."
        style={{
          width: "600px",
          height: "120px",
          marginTop: "20px",
          padding: "10px",
          background: "#2d2d2d",
          color: "white",
          border: "1px solid #555",
        }}
      />

      <br />

      {/* ---------------- Action Buttons ---------------- */}
      <button
        onClick={handleAnalyze}
        style={{
          marginTop: "10px",
          marginRight: "10px",
          padding: "8px 15px",
          background: "#0066ff",
          border: "none",
          color: "white",
          borderRadius: "6px",
        }}
      >
        Analyze Query
      </button>

      <button
        onClick={handleExecute}
        style={{
          padding: "8px 15px",
          background: "#009944",
          border: "none",
          color: "white",
          borderRadius: "6px",
        }}
      >
        Run Query
      </button>

      {/* ---------------- Debug Plan Tree ---------------- */}
      {plan && (
        <div style={{ marginTop: "30px" }}>
          <h3>Parsed Plan Tree (Debug)</h3>
          <pre
            style={{
              background: "#111",
              padding: "20px",
              width: "600px",
              overflowX: "auto",
            }}
          >
            {JSON.stringify(plan, null, 2)}
          </pre>
        </div>
      )}

      {/* ---------------- Graph ---------------- */}
      {plan && (
        <div
          style={{
            height: "500px",
            width: "900px",
            marginTop: "20px",
            border: "1px solid #444",
          }}
        >
          <PlanGraph plan={plan} />
        </div>
      )}

      {/* ---------------- Query Execution Output ---------------- */}
      {result && (
        <div style={{ marginTop: "40px" }}>
          <h3>Query Output</h3>

          {result.success && (
            <table style={{ width: "600px", borderCollapse: "collapse" }}>
              <thead>
                <tr>
                  {result.columns?.map((c: string) => (
                    <th
                      key={c}
                      style={{ border: "1px solid white", padding: "5px" }}
                    >
                      {c}
                    </th>
                  ))}
                </tr>
              </thead>

              <tbody>
                {result.rows?.map((row: any[], idx: number) => (
                  <tr key={idx}>
                    {row.map((v, i) => (
                      <td
                        key={i}
                        style={{ border: "1px solid white", padding: "5px" }}
                      >
                        {String(v)}
                      </td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      )}
    </div>
  );
}

export default App;

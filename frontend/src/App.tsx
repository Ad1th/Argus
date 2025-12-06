import { useState } from "react";
import PlanGraph from "./components/PlanGraph";

function App() {
  const [query, setQuery] = useState("");
  const [planTree, setPlanTree] = useState<any>(null);
  const [error, setError] = useState<string | null>(null);

  const analyzeQuery = async () => {
    setError(null);
    setPlanTree(null);

    try {
      const res = await fetch("http://localhost:8000/api/sql/parse-plan", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ query }),
      });

      const data = await res.json();
      console.log("API response:", data);

      if (!data.success) {
        setError(data.error || "Query failed");
        return;
      }

      setPlanTree(data.plan_tree);
    } catch (e: any) {
      setError("Could not connect to backend");
    }
  };

  return (
    <div
      style={{
        background: "#1e1e1e",
        minHeight: "100vh",
        padding: "40px",
        color: "white",
      }}
    >
      <h1 style={{ fontSize: "32px", marginBottom: "20px" }}>
        AetherQuery — Plan Visualizer
      </h1>

      {/* Query Input */}
      <textarea
        placeholder="Enter SQL query..."
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        style={{
          width: "60%",
          height: "120px",
          padding: "10px",
          borderRadius: "8px",
          background: "#2b2b2b",
          border: "1px solid #555",
          color: "white",
        }}
      />

      <br />
      <button
        onClick={analyzeQuery}
        style={{
          marginTop: "10px",
          padding: "8px 16px",
          background: "#333",
          borderRadius: "6px",
          color: "white",
          border: "1px solid #777",
          cursor: "pointer",
        }}
      >
        Analyze Query
      </button>

      {/* Error Message */}
      {error && (
        <div style={{ marginTop: "20px", color: "red" }}>❌ {error}</div>
      )}

      {/* Debug: Show JSON tree */}
      {planTree && (
        <div style={{ marginTop: "30px" }}>
          <h3>Parsed Plan Tree (Debug)</h3>
          <pre
            style={{
              background: "#111",
              padding: "15px",
              borderRadius: "8px",
              color: "#0f0",
              maxHeight: "300px",
              overflowY: "auto",
              width: "60%",
            }}
          >
            {JSON.stringify(planTree, null, 2)}
          </pre>

          {/* Graph Renderer */}
          <div style={{ height: "600px", marginTop: "40px" }}>
            <PlanGraph plan={planTree} />
          </div>
        </div>
      )}
    </div>
  );
}

export default App;

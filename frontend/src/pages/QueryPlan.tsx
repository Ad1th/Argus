import { useState } from "react";
import PlanGraph from "../components/PlanGraph";

const backend = import.meta.env.VITE_API_BASE_URL ?? "http://127.0.0.1:8000";

export default function QueryPlanPage() {
  const [query, setQuery] = useState("");
  const [plan, setPlan] = useState(null);

  async function analyzeQuery() {
    const res = await fetch(`${backend}/api/sql/parse-plan`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ query }),
    });

    const data = await res.json();
    setPlan(data.plan_tree ?? null);
  }

  return (
    <div style={{ padding: 20 }}>
      <h1>AetherQuery — Plan Visualizer</h1>

      <textarea
        placeholder="Enter SQL query..."
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        style={{ width: "100%", height: 120, marginBottom: 20 }}
      />

      <button onClick={analyzeQuery} style={{ padding: "8px 16px" }}>
        Analyze Query
      </button>

      {plan && (
        <div style={{ height: 500, marginTop: 20 }}>
          <PlanGraph plan={plan} />
        </div>
      )}
    </div>
  );
}

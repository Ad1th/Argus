import { useState } from "react";
import PlanGraph from "../components/PlanGraph";
import { planToFlow } from "../utils/planToFlow";

export default function QueryPlanPage() {
  const [query, setQuery] = useState("");
  const [flow, setFlow] = useState(null);

  async function analyzeQuery() {
    const res = await fetch("http://localhost:8000/api/sql/parse-plan", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ query }),
    });

    const data = await res.json();
    const { nodes, edges } = planToFlow(data.plan_tree);

    setFlow({ nodes, edges });
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

      {flow && <PlanGraph nodes={flow.nodes} edges={flow.edges} />}
    </div>
  );
}

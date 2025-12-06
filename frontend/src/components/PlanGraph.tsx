import React, { useCallback } from "react";
import ReactFlow, {
  Background,
  Controls,
  MiniMap,
  useNodesState,
  useEdgesState,
  addEdge,
} from "reactflow";

import "reactflow/dist/style.css";

// Convert your plan JSON into graph nodes & edges
function buildGraph(plan: any) {
  const nodes: any[] = [];
  const edges: any[] = [];

  function dfs(node: any, parentId: string | null, depth = 0, index = 0) {
    const nodeId = `${node.type}-${Math.random().toString(36).slice(2, 8)}`;

    nodes.push({
      id: nodeId,
      data: { label: node.type },
      position: { x: depth * 250, y: index * 120 },
      style: {
        padding: 10,
        borderRadius: 8,
        background: "#222",
        color: "white",
        border: "1px solid #555",
      },
    });

    if (parentId) {
      edges.push({
        id: `e-${parentId}-${nodeId}`,
        source: parentId,
        target: nodeId,
      });
    }

    // recursively go through children
    node.children?.forEach((child: any, i: number) => {
      dfs(child, nodeId, depth + 1, i);
    });
  }

  dfs(plan, null);
  return { nodes, edges };
}

export default function PlanGraph({ plan }: { plan: any }) {
  const { nodes: initialNodes, edges: initialEdges } = buildGraph(plan);

  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);

  const onConnect = useCallback(
    (params: any) => setEdges((eds) => addEdge(params, eds)),
    []
  );

  return (
    <div style={{ width: "100%", height: "100%" }}>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        fitView
      >
        <Background />
        <Controls />
        <MiniMap />
      </ReactFlow>
    </div>
  );
}

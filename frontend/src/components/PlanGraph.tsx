import React, { useCallback } from "react";
import ReactFlow, { Background, Controls, MiniMap } from "reactflow";
import "reactflow/dist/style.css";

interface NodeType {
  type: string;
  columns?: string[];
  aggregates?: string[];
  children?: NodeType[];
}

function PlanGraph({ plan }: { plan: NodeType }) {
  let nodes: any[] = [];
  let edges: any[] = [];

  let idCounter = 1;

  const dfs = (node: NodeType, parentId: string | null): string => {
    const id = `n${idCounter++}`;

    nodes.push({
      id,
      position: { x: Math.random() * 600, y: Math.random() * 600 },
      data: { label: node.type },
      style: {
        padding: 10,
        border: "1px solid #ddd",
        borderRadius: 5,
        background: "#222",
        color: "white",
      },
    });

    if (parentId) {
      edges.push({
        id: `e-${parentId}-${id}`,
        source: parentId,
        target: id,
      });
    }

    if (node.children) {
      node.children.forEach((child: NodeType) => dfs(child, id));
    }

    return id;
  };

  dfs(plan, null);

  return (
    <ReactFlow nodes={nodes} edges={edges} fitView>
      <Background />
      <MiniMap />
      <Controls />
    </ReactFlow>
  );
}

export default PlanGraph;

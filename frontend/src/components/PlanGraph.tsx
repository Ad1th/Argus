import React from "react";
import ReactFlow, { Background, Controls, MiniMap } from "reactflow";
import "reactflow/dist/style.css";

export default function PlanGraph({ nodes, edges }: any) {
  return (
    <div style={{ width: "100%", height: "600px" }}>
      <ReactFlow nodes={nodes} edges={edges} fitView>
        <Background variant="dots" gap={12} />
        <Controls />
        <MiniMap />
      </ReactFlow>
    </div>
  );
}

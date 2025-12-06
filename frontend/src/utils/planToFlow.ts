import { v4 as uuid } from "uuid";

export function planToFlow(plan: any) {
  const nodes: any[] = [];
  const edges: any[] = [];

  function traverse(node: any, parentId?: string) {
    const id = uuid();

    nodes.push({
      id,
      data: {
        label: node.type,
        columns: node.columns,
        aggregates: node.aggregates,
        rows: node.rows,
      },
      position: { x: 0, y: 0 },
    });

    if (parentId) {
      edges.push({
        id: uuid(),
        source: parentId,
        target: id,
      });
    }

    if (node.children) {
      node.children.forEach((c: any) => traverse(c, id));
    }
  }

  traverse(plan);

  return { nodes, edges };
}

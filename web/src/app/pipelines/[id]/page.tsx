import "@xyflow/react/dist/style.css";
import type { useEdgesState, useNodesState } from "@xyflow/react";
import type { Pipeline } from "~/lib/dtos";
import { Editor } from "./_components/editor";

type Params = Promise<{ id: string }>;

export default async function PipelineDetails(props: { params: Params }) {
	const params = await props.params;

	const data = await fetch(
		`http://192.168.0.2:8000/api/v0/pipelines/${params.id}`,
	);
	const pipeline: Pipeline = await data.json();

	const nodes: Parameters<typeof useNodesState>[0] = pipeline.nodes.map(
		(node) => ({
			id: node.id,
			position: node.coords,
			data: {
				label: node.id,
			},
		}),
	);

	const edges: Parameters<typeof useEdgesState>[0] = pipeline.connections.map(
		(connection) => ({
			id: connection.id,
			source: connection.from_node_id,
			target: connection.to_node_id,
		}),
	);

	return (
		<div className="grid grid-rows-[20px_1fr_20px] items-center justify-items-center min-h-screen p-8 pb-20 gap-16 sm:p-20 font-[family-name:var(--font-geist-sans)]">
			<h1 className="text-3xl font-bold text-center">
				Pipeline {pipeline.name}
			</h1>

			<Editor
				nodes={nodes}
				edges={edges}
				participants={pipeline.participants}
			/>
		</div>
	);
}

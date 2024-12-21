import "@xyflow/react/dist/style.css";
import {
	ReactFlowProvider,
	type useEdgesState,
	type useNodesState,
} from "@xyflow/react";
import { Link as LinkIcon } from "lucide-react";
import Link from "next/link";
import { Button } from "~/components/ui/button";
import { getSessionToken } from "~/lib/auth";
import { type Node, NodeType, type Pipeline } from "~/lib/dtos";
import { Editor } from "./_components/editor";

type Params = Promise<{ id: string }>;

export default async function PipelineDetails(props: { params: Params }) {
	const params = await props.params;
	let pipeline: Pipeline;
	let availableNodes: Node[] = [];

	const sessionToken = await getSessionToken();

	try {
		const data = await fetch(
			`http://192.168.0.2:8000/api/v0/pipelines/${params.id}?withParticipants=includeMyself`,
			{
				headers: {
					Accept: "application/json",
					"Content-Type": "application/json",
					Authorization: `Bearer ${sessionToken}`,
				},
			},
		);

		pipeline = await data.json();
	} catch (error) {
		console.error(error);
		return <div>Failed to fetch pipeline</div>;
	}

	try {
		const data = await fetch("http://192.168.0.2:8000/api/v0/nodes", {
			headers: {
				Accept: "application/json",
				"Content-Type": "application/json",
				Authorization: `Bearer ${sessionToken}`,
			},
		});

		availableNodes = await data.json();
	} catch (error) {
		console.error(error);
		return <div>Failed to fetch nodes</div>;
	}

	const nodes: Parameters<typeof useNodesState>[0] = pipeline.nodes.map(
		(pipeline_node) => {
			const node = availableNodes.find(
				(available_node) => available_node.id === pipeline_node.node_id,
			);

			return {
				id: pipeline_node.id,
				position: pipeline_node.coords,
				type: NodeType.Task,
				data: {
					name: `${node?.name}:${pipeline_node.node_version}`,
					config: node?.config,
				},
			};
		},
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
				<Button>
					<Link href="/pipelines">
						<div className="flex items-center space-x-2">
							<LinkIcon />
							<span>Back</span>
						</div>
					</Link>
				</Button>
				{pipeline.name}
			</h1>

			<ReactFlowProvider>
				<Editor
					nodes={nodes}
					edges={edges}
					participants={pipeline.participants ?? []}
					availableNodes={availableNodes}
				/>
			</ReactFlowProvider>
		</div>
	);
}

import "@xyflow/react/dist/style.css";
import {
	ReactFlowProvider,
	type useEdgesState,
	type useNodesState,
} from "@xyflow/react";
import { Link as LinkIcon } from "lucide-react";
import Link from "next/link";
import { Button } from "~/components/ui/button";
import { env } from "~/env";
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
		const response = await fetch(
			`http://localhost:8000/api/v0/pipelines/${params.id}?withParticipants=includeMyself`,
			{
				headers: {
					Accept: "application/json",
					"Content-Type": "application/json",
					Authorization: `Bearer ${sessionToken}`,
				},
			},
		);

		console.debug(response);

		pipeline = await response.json();
	} catch (error) {
		console.error(error);
		return <div>Failed to fetch pipeline</div>;
	}

	try {
		const data = await fetch(`${env.API_URL}/nodes`, {
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
					inputs: pipeline_node.inputs.map((input) => ({
						...input,
						controlled: pipeline.nodes.some((node) => {
							return node.outputs.some((output) => {
								return pipeline.connections.some((connection) => {
									return (
										connection.from_pipeline_node_output_id === output.id &&
										connection.to_pipeline_node_input_id === input.id
									);
								});
							});
						}),
					})),
					outputs: pipeline_node.outputs,
				},
			};
		},
	);

	nodes.push({
		id: pipeline.trigger.id,
		position: {
			x: -154,
			y: -112,
		},
		type: NodeType.Trigger,
		data: {
			name: "Trigger",
			pipelineId: pipeline.id,
			config: pipeline.trigger.config,
		},
	});

	const edges: Parameters<typeof useEdgesState>[0] = pipeline.connections.map(
		(connection) => ({
			id: connection.id,
			source:
				pipeline.nodes.find((node) => {
					return node.outputs.some((output) => {
						return output.id === connection.from_pipeline_node_output_id;
					});
				})?.id ?? "",
			target:
				pipeline.nodes.find((node) => {
					return node.inputs.some((input) => {
						return input.id === connection.to_pipeline_node_input_id;
					});
				})?.id ?? "",
			animated: true,
			deletable: true,
			focusable: true,
			sourceHandle: connection.from_pipeline_node_output_id,
			targetHandle: connection.to_pipeline_node_input_id,
			selectable: true,
		}),
	);

	edges.push({
		id: pipeline.trigger.id,
		source: pipeline.trigger.id,
		target:
			pipeline.nodes.find((node) => {
				return node.trigger_id === pipeline.trigger.id;
			})?.id ?? "",
		animated: true,
		deletable: true,
		focusable: true,
		selectable: true,
	});

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

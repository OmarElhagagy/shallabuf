import "@xyflow/react/dist/style.css";
import type { useEdgesState, useNodesState } from "@xyflow/react";
import { Link as LinkIcon } from "lucide-react";
import Link from "next/link";
import { Button } from "~/components/ui/button";
import { getSessionToken } from "~/lib/auth";
import type { Pipeline } from "~/lib/dtos";
import { Editor } from "./_components/editor";

type Params = Promise<{ id: string }>;

export default async function PipelineDetails(props: { params: Params }) {
	const params = await props.params;
	let pipeline: Pipeline;

	const sessionToken = await getSessionToken();

	try {
		const data = await fetch(
			`http://192.168.0.2:8000/api/v0/pipelines/${params.id}`,
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
				<Button>
					<Link href="/pipelines">
						<div className="flex items-center space-x-2">
							<LinkIcon />
							<span>Back</span>
						</div>
					</Link>
				</Button>
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

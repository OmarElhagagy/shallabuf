"use client";
import {
	type OnConnect,
	ReactFlow,
	addEdge,
	useEdgesState,
	useNodesState,
} from "@xyflow/react";
import React, { useCallback, useEffect } from "react";
import {
	WsAction,
	type WsActionMessage,
	type WsActionUpdateNode,
} from "~/app/_ws/ws-action";
import { useWs } from "~/app/_ws/ws-context";
import { updatePipelineNode } from "~/app/actions/update-node";

export interface EditorProps {
	nodes: Parameters<typeof useNodesState>[0];
	edges: Parameters<typeof useEdgesState>[0];
}

export const Editor = (props: EditorProps) => {
	const { ws } = useWs();
	const [nodes, setNodes, onNodesChange] = useNodesState(props.nodes);
	const [edges, setEdges, onEdgesChange] = useEdgesState(props.edges);

	const onConnect: OnConnect = useCallback(
		(params) => setEdges((eds) => addEdge(params, eds)),
		[setEdges],
	);

	const onWsMessage = useCallback(
		(event: MessageEvent) => {
			const message: WsActionMessage<WsActionUpdateNode> = JSON.parse(
				event.data,
			);

			setNodes((nodes) =>
				nodes.map((node) => {
					if (node.id === message.payload.id) {
						return {
							...node,
							position: message.payload.coords,
						};
					}

					return node;
				}),
			);
		},
		[setNodes],
	);

	useEffect(() => {
		ws?.addEventListener("message", onWsMessage);
		return () => ws?.removeEventListener("message", onWsMessage);
	}, [ws, onWsMessage]);

	const broadcastNodePosition: NonNullable<
		Parameters<typeof ReactFlow>[0]["onNodeDrag"]
	> = useCallback(
		(_event, node) => {
			ws?.send(
				JSON.stringify({
					action: WsAction.UpdateNode,
					payload: {
						id: node.id,
						coords: {
							x: node.position.x,
							y: node.position.y,
						},
					},
				}),
			);
		},
		[ws],
	);

	const updateNodePosition: NonNullable<
		Parameters<typeof ReactFlow>[0]["onNodeDragStop"]
	> = useCallback(async (_event, node) => {
		await updatePipelineNode({
			id: node.id,
			coords: {
				x: node.position.x,
				y: node.position.y,
			},
		});
	}, []);

	return (
		<div className="w-full h-full">
			<ReactFlow
				nodes={nodes}
				edges={edges}
				onNodesChange={onNodesChange}
				onEdgesChange={onEdgesChange}
				onConnect={onConnect}
				onNodeDragStop={updateNodePosition}
				onNodeDrag={broadcastNodePosition}
				fitView
				proOptions={{
					hideAttribution: true,
				}}
			/>
		</div>
	);
};

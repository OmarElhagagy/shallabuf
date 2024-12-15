"use client";
import {
	type OnConnect,
	ReactFlow,
	addEdge,
	useEdgesState,
	useNodesState,
} from "@xyflow/react";
import React, { useCallback, useEffect, useState } from "react";
import {
	WsAction,
	type WsActionAddEditorParticipant,
	type WsActionMessage,
	type WsActionPayload,
	type WsActionRemoveEditorParticipant,
	isAddEditorParticipant,
	isNodeUpdate,
	isRemoveEditorParticipant,
} from "~/app/_ws/ws-action";
import { useWs } from "~/app/_ws/ws-context";
import { updatePipelineNodeAction } from "~/app/actions/update-pipeline-node";

export interface Participant {
	id: string;
	name: string;
}

export interface EditorProps {
	nodes: Parameters<typeof useNodesState>[0];
	edges: Parameters<typeof useEdgesState>[0];
	participants: Participant[];
}

export const Editor = (props: EditorProps) => {
	const { ws } = useWs();
	const [nodes, setNodes, onNodesChange] = useNodesState(props.nodes);
	const [edges, setEdges, onEdgesChange] = useEdgesState(props.edges);
	const [participants, setParticipants] = useState(props.participants);

	const onConnect: OnConnect = useCallback(
		(params) => setEdges((eds) => addEdge(params, eds)),
		[setEdges],
	);

	const onWsMessage = useCallback(
		(event: MessageEvent) => {
			const message: WsActionMessage<WsActionPayload> = JSON.parse(event.data);

			console.log("Received message", message);

			if (isNodeUpdate(message)) {
				console.log(
					"Node update",
					message.payload.node_id,
					message.payload.coords,
				);

				setNodes((nodes) =>
					nodes.map((node) => {
						if (node.id === message.payload.node_id) {
							return {
								...node,
								position: message.payload.coords,
							};
						}

						return node;
					}),
				);
			} else if (isAddEditorParticipant(message)) {
				setParticipants((participants) => [
					...participants,
					{
						id: message.sender_id ?? message.payload.user_id,
						name: message.payload.username,
					},
				]);
			} else if (isRemoveEditorParticipant(message)) {
				setParticipants((participants) =>
					participants.filter(
						(participant) => participant.id !== message.payload.user_id,
					),
				);
			}
		},
		[setNodes],
	);

	useEffect(() => {
		if (!ws || ws.readyState !== ws.OPEN) {
			return;
		}

		const user_id = Math.random().toString(36).substring(2, 10);

		const data: WsActionMessage<WsActionAddEditorParticipant> = {
			action: WsAction.AddEditorParticipant,
			sender_id: null,
			payload: {
				pipeline_id: "1",
				user_id,
				username: `user_${Math.random().toString(36).substring(2, 5)}`,
			},
		};

		ws.send(JSON.stringify(data));

		return () => {
			const data: WsActionMessage<WsActionRemoveEditorParticipant> = {
				action: WsAction.RemoveEditorParticipant,
				sender_id: null,
				payload: {
					pipeline_id: "1",
					user_id,
				},
			};

			ws.send(JSON.stringify(data));
		};
	}, [ws, ws?.readyState]);

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
					sender_id: null,
					payload: {
						node_id: node.id,
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
		await updatePipelineNodeAction({
			id: node.id,
			coords: {
				x: node.position.x,
				y: node.position.y,
			},
		});
	}, []);

	return (
		<div className="w-full h-full">
			<div className="p-4 bg-white shadow rounded">
				<h2 className="text-xl font-bold mb-2">Participants</h2>

				<ul className="list-disc pl-5">
					{participants.map((participant) => (
						<li key={participant.id} className="py-1">
							{participant.name}
						</li>
					))}
				</ul>
			</div>

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

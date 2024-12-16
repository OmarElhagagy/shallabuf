"use client";
import {
	type OnConnect,
	ReactFlow,
	addEdge,
	useEdgesState,
	useNodesState,
} from "@xyflow/react";
import { useParams } from "next/navigation";
import React, { useCallback, useEffect } from "react";
import { useShallow } from "zustand/react/shallow";
import { updatePipelineNodeAction } from "~/app/actions/update-pipeline-node";
import { useWsStore } from "~/contexts/ws-store-context";
import type { PipelineParticipant } from "~/lib/dtos";

export interface EditorProps {
	nodes: Parameters<typeof useNodesState>[0];
	edges: Parameters<typeof useEdgesState>[0];
	participants: PipelineParticipant[];
}

export const Editor = (props: EditorProps) => {
	const [nodes, , onNodesChange] = useNodesState(props.nodes);
	const [edges, setEdges, onEdgesChange] = useEdgesState(props.edges);
	const params = useParams();
	const pipelineId = params.id as string;

	const [
		participants,
		initPipelineParticipants,
		enterPipelineEditor,
		leavePipelineEditor,
	] = useWsStore(
		useShallow((state) => [
			state.pipelinesParticipants[pipelineId] ?? {},
			state.initPipelineParticipants,
			state.enterPipelineEditor,
			state.leavePipelineEditor,
		]),
	);

	const onConnect: OnConnect = useCallback(
		(params) => setEdges((eds) => addEdge(params, eds)),
		[setEdges],
	);

	useEffect(() => {
		initPipelineParticipants(
			pipelineId,
			props.participants.reduce(
				(acc, participant) => {
					acc[participant.id] = participant.name;
					return acc;
				},
				{} as Record<string, string>,
			),
		);
	}, [initPipelineParticipants, props.participants, pipelineId]);

	useEffect(() => {
		enterPipelineEditor(pipelineId);

		return () => {
			leavePipelineEditor(pipelineId);
		};
	}, [enterPipelineEditor, leavePipelineEditor, pipelineId]);

	// const onWsMessage = useCallback(
	// 	(event: MessageEvent) => {
	// 		const message: WsActionMessage<WsActionPayload> = JSON.parse(event.data);

	// 		console.log("Received message", message);

	// 		if (isNodeUpdate(message)) {
	// 			console.log(
	// 				"Node update",
	// 				message.payload.node_id,
	// 				message.payload.coords,
	// 			);

	// 			setNodes((nodes) =>
	// 				nodes.map((node) => {
	// 					if (node.id === message.payload.node_id) {
	// 						return {
	// 							...node,
	// 							position: message.payload.coords,
	// 						};
	// 					}

	// 					return node;
	// 				}),
	// 			);
	// 		} else if (isAddEditorParticipant(message)) {
	// 			setParticipants((participants) => [
	// 				...participants,
	// 				{
	// 					id: message.sender_id ?? message.payload.user_id,
	// 					name: message.payload.username,
	// 				},
	// 			]);
	// 		} else if (isRemoveEditorParticipant(message)) {
	// 			setParticipants((participants) =>
	// 				participants.filter(
	// 					(participant) => participant.id !== message.payload.user_id,
	// 				),
	// 			);
	// 		}
	// 	},
	// 	[setNodes],
	// );

	// useEffect(() => {
	// 	if (!ws || ws.readyState !== ws.OPEN) {
	// 		return;
	// 	}

	// 	const user_id = Math.random().toString(36).substring(2, 10);

	// 	const data: WsActionMessage<WsActionAddEditorParticipant> = {
	// 		action: WsAction.AddEditorParticipant,
	// 		sender_id: null,
	// 		payload: {
	// 			pipeline_id: "1",
	// 			user_id,
	// 			username: `user_${Math.random().toString(36).substring(2, 5)}`,
	// 		},
	// 	};

	// 	ws.send(JSON.stringify(data));

	// 	return () => {
	// 		const data: WsActionMessage<WsActionRemoveEditorParticipant> = {
	// 			action: WsAction.RemoveEditorParticipant,
	// 			sender_id: null,
	// 			payload: {
	// 				pipeline_id: "1",
	// 				user_id,
	// 			},
	// 		};

	// 		ws.send(JSON.stringify(data));
	// 	};
	// }, [ws, ws?.readyState]);

	// useEffect(() => {
	// 	ws?.addEventListener("message", onWsMessage);
	// 	return () => ws?.removeEventListener("message", onWsMessage);
	// }, [ws, onWsMessage]);

	// const broadcastNodePosition: NonNullable<
	// 	Parameters<typeof ReactFlow>[0]["onNodeDrag"]
	// > = useCallback(
	// 	(_event, node) => {
	// 		ws?.send(
	// 			JSON.stringify({
	// 				action: WsAction.UpdateNode,
	// 				sender_id: null,
	// 				payload: {
	// 					node_id: node.id,
	// 					coords: {
	// 						x: node.position.x,
	// 						y: node.position.y,
	// 					},
	// 				},
	// 			}),
	// 		);
	// 	},
	// 	[ws],
	// );

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
					{Object.entries(participants).map(([id, name]) => (
						<li key={id} className="py-1">
							{name}
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
				// onNodeDrag={broadcastNodePosition}
				fitView
				proOptions={{
					hideAttribution: true,
				}}
			/>
		</div>
	);
};

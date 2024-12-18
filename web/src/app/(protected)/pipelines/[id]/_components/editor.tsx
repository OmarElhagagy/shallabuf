"use client";
import {
	type OnConnect,
	ReactFlow,
	addEdge,
	useEdgesState,
	useNodesState,
	useReactFlow,
} from "@xyflow/react";
import { MousePointer2 } from "lucide-react";
import { useParams } from "next/navigation";
import React, { type MouseEvent, useCallback, useEffect, useRef } from "react";
import { useShallow } from "zustand/react/shallow";
import { updatePipelineNodeAction } from "~/app/actions/update-pipeline-node";
import { useWsStore } from "~/contexts/ws-store-context";
import type { PipelineParticipant } from "~/lib/dtos";
import type { WsStoreState } from "~/stores/ws-store";

export interface EditorProps {
	nodes: Parameters<typeof useNodesState>[0];
	edges: Parameters<typeof useEdgesState>[0];
	participants: PipelineParticipant[];
}

export const Editor = (props: EditorProps) => {
	const [nodes, setNodes, onNodesChange] = useNodesState(props.nodes);
	const [edges, setEdges, onEdgesChange] = useEdgesState(props.edges);
	const params = useParams();
	const pipelineId = params.id as string;

	const [
		participants,
		subscribeForNodeUpdates,
		initPipelineParticipants,
		enterPipelineEditor,
		leavePipelineEditor,
		updateCursorPosition,
		updateNodePosition,
	] = useWsStore(
		useShallow((state) => [
			state.pipelinesParticipants[pipelineId] ?? {},
			state.subscribeForNodeUpdates,
			state.initPipelineParticipants,
			state.enterPipelineEditor,
			state.leavePipelineEditor,
			state.updateCursorPosition,
			state.updateNodePosition,
		]),
	);

	useEffect(() => {
		return subscribeForNodeUpdates(pipelineId, (update) => {
			setNodes((nodes) => {
				const node = nodes.find((node) => node.id === update.nodeId);

				if (!node) {
					return nodes;
				}

				return [
					...nodes.filter((node) => node.id !== update.nodeId),
					{
						...node,
						position: update.nodePosition,
					},
				];
			});
		});
	}, [pipelineId, setNodes, subscribeForNodeUpdates]);

	const onConnect: OnConnect = useCallback(
		(params) => setEdges((eds) => addEdge(params, eds)),
		[setEdges],
	);

	useEffect(() => {
		initPipelineParticipants(
			pipelineId,
			props.participants.reduce(
				(acc, participant) => {
					acc[participant.id] = {
						username: participant.name,
					};
					return acc;
				},
				{} as WsStoreState["pipelinesParticipants"][string],
			),
		);
	}, [initPipelineParticipants, props.participants, pipelineId]);

	useEffect(() => {
		enterPipelineEditor(pipelineId);

		return () => {
			leavePipelineEditor(pipelineId);
		};
	}, [enterPipelineEditor, leavePipelineEditor, pipelineId]);

	const broadcastNodePosition: NonNullable<
		Parameters<typeof ReactFlow>[0]["onNodeDrag"]
	> = useCallback(
		(_event, node) => {
			updateNodePosition(pipelineId, node.id, node.position);
		},
		[pipelineId, updateNodePosition],
	);

	const saveNodePosition: NonNullable<
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

	const { getViewport } = useReactFlow();
	const viewportContainer = useRef<HTMLDivElement>(null);

	const broadcastCursorPosition = useCallback(
		(event: MouseEvent<HTMLDivElement>) => {
			const rect = viewportContainer.current?.getBoundingClientRect();

			if (!rect) {
				return;
			}

			const viewport = getViewport();

			const adjustedX =
				(event.clientX - rect.left - viewport.x) / viewport.zoom;
			const adjustedY = (event.clientY - rect.top - viewport.y) / viewport.zoom;

			updateCursorPosition(pipelineId, {
				x: adjustedX,
				y: adjustedY,
			});
		},
		[getViewport, updateCursorPosition, pipelineId],
	);

	return (
		<div className="w-full h-full relative">
			<div className="p-4 bg-white shadow rounded">
				<h2 className="text-xl font-bold mb-2">Participants</h2>

				<ul className="list-disc pl-5">
					{Object.entries(participants).map(
						([id, { username, cursorPosition }]) => (
							<li key={id} className="py-1">
								{username}

								{cursorPosition && (
									<span className="text-xs text-gray-500 ml-2">
										({cursorPosition.x}, {cursorPosition.y})
									</span>
								)}
							</li>
						),
					)}
				</ul>
			</div>

			<div
				ref={viewportContainer}
				className="relative h-full w-full border border-gray-200 mt-4"
			>
				<ReactFlow
					onMouseMove={broadcastCursorPosition}
					nodes={nodes}
					edges={edges}
					onNodesChange={onNodesChange}
					onEdgesChange={onEdgesChange}
					onConnect={onConnect}
					onNodeDragStop={saveNodePosition}
					onNodeDrag={broadcastNodePosition}
					fitView
					proOptions={{
						hideAttribution: true,
					}}
				/>

				{Object.entries(participants).map(
					([id, { username, cursorPosition }]) => {
						if (!cursorPosition) {
							return null;
						}

						const viewport = getViewport();

						return (
							<div
								key={id}
								className="absolute pointer-events-none z-10 -translate-x-1/2 -translate-y-1/2"
								style={{
									left: cursorPosition.x * viewport.zoom + viewport.x,
									top: cursorPosition.y * viewport.zoom + viewport.y,
								}}
							>
								<MousePointer2 />
								<span className="username">{username}</span>
							</div>
						);
					},
				)}
			</div>
		</div>
	);
};

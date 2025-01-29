"use client";
import {
	DndContext,
	DragOverlay,
	type DragStartEvent,
	type UniqueIdentifier,
} from "@dnd-kit/core";
import {
	Background,
	BackgroundVariant,
	type Edge,
	type OnConnect,
	Panel,
	ReactFlow,
	type ReactFlowProps,
	addEdge,
	useEdgesState,
	useNodesState,
	useReactFlow,
} from "@xyflow/react";
import { MousePointer2 } from "lucide-react";
import { useParams } from "next/navigation";
import { useQueryState } from "nuqs";
import React, {
	type MouseEvent,
	useCallback,
	useEffect,
	useRef,
	useState,
} from "react";
import { useShallow } from "zustand/react/shallow";
import { createPipelineNodeAction } from "~/actions/create-pipeline-node";
import { createPipelineNodeConnectionAction } from "~/actions/create-pipeline-node-connection";
import { createPipelineTriggerConnectionAction } from "~/actions/create-pipeline-trigger-connection";
import { updatePipelineNodeAction } from "~/actions/update-pipeline-node";
import { updatePipelineTriggerAction } from "~/actions/update-pipeline-trigger";
import { useWsStore } from "~/contexts/ws-store-context";
import { env } from "~/env";
import {
	type Node,
	NodeType,
	type PipelineExecNotification,
	type PipelineNode,
	type PipelineNodeConnection,
	type PipelineParticipant,
} from "~/lib/dtos";
import type { WsStoreState } from "~/stores/ws-store";
import { Dropzone } from "./dropzone";
import { NodeItem } from "./node-item";
import { TaskNode } from "./task-node";
import { TriggerNode } from "./trigger-node";

export interface EditorProps {
	nodes: Parameters<typeof useNodesState>[0];
	edges: Parameters<typeof useEdgesState>[0];
	participants: PipelineParticipant[];
	availableNodes: Node[];
}

const nodeTypes: ReactFlowProps["nodeTypes"] = {
	task: TaskNode,
	trigger: TriggerNode,
};

export const Editor = (props: EditorProps) => {
	const [nodes, setNodes, onNodesChange] = useNodesState(props.nodes);
	const [edges, setEdges, onEdgesChange] = useEdgesState(props.edges);
	const params = useParams();
	const pipelineId = params.id as string;
	const [execId, setExecId] = useQueryState("exec");

	useEffect(() => {
		if (execId) {
			const eventSource = new EventSource(
				`${env.NEXT_PUBLIC_API_URL}/pipeline-execs/${execId}/events`,
			);

			eventSource.onmessage = (event) => {
				const notification: PipelineExecNotification = JSON.parse(event.data);

				if (notification.type === "pipeline") {
					setNodes((nodes) => {
						return nodes.map((node) => {
							if (node.type === NodeType.Trigger) {
								return {
									...node,
									data: {
										...node.data,
										execStatus: notification.data.status,
									},
								};
							}

							return node;
						});
					});

					if (
						notification.data.status === "completed" ||
						notification.data.status === "failed"
					) {
						setExecId(null);
						eventSource.close();
					}
				}

				if (notification.type === "node") {
					setNodes((nodes) => {
						return nodes.map((node) => {
							if (node.id === notification.data.pipelineNodeId) {
								return {
									...node,
									data: {
										...node.data,
										result: notification.data.result,
										execStatus: notification.data.status,
									},
								};
							}

							return node;
						});
					});
				}
			};

			eventSource.onerror = (event) => {
				setExecId(null);
				console.error(event);
			};
		}
	}, [execId, setNodes, setExecId]);

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
		async (params) => {
			const commonEdgeParams: Partial<Edge> = {
				animated: true,
				deletable: true,
				focusable: true,
				selectable: true,
			};

			if (
				nodes.find((node) => node.id === params.source)?.type ===
				NodeType.Trigger
			) {
				const pipelineNode: PipelineNode =
					await createPipelineTriggerConnectionAction({
						triggerId: params.source,
						nodeId: params.target,
					});

				setNodes((nodes) => {
					return nodes.map((node) => {
						if (node.id === pipelineNode.id) {
							return {
								...node,
								data: {
									...node.data,
									triggerId: pipelineNode.triggerId,
								},
							};
						}

						return node;
					});
				});

				setEdges((eds) => {
					return addEdge(
						{
							...commonEdgeParams,
							id: pipelineNode.triggerId ?? "",
							source: pipelineNode.triggerId ?? "",
							target: pipelineNode.id,
						},
						eds,
					);
				});
			} else {
				const connection: PipelineNodeConnection =
					await createPipelineNodeConnectionAction({
						fromNodeId: params.source,
						toNodeId: params.target,
					});

				setEdges((eds) => {
					return addEdge(
						{
							...commonEdgeParams,
							id: connection.id,
							source: params.source,
							target: params.target,
							sourceHandle: connection.fromPipelineNodeOutputId,
							targetHandle: connection.toPipelineNodeInputId,
						},
						eds,
					);
				});
			}
		},
		[nodes, setEdges, setNodes],
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
		if (node.type === NodeType.Trigger) {
			await updatePipelineTriggerAction({
				id: node.id,
				coords: {
					x: node.position.x,
					y: node.position.y,
				},
			});
		} else {
			await updatePipelineNodeAction({
				id: node.id,
				coords: {
					x: node.position.x,
					y: node.position.y,
				},
			});
		}
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

	const [draggableNodeItem, setDraggableNodeItem] =
		useState<UniqueIdentifier>("");

	const onDragStart = useCallback((event: DragStartEvent) => {
		setDraggableNodeItem(event.active.id);
	}, []);

	const onDragEnd = useCallback(async () => {
		const pipelineNode: PipelineNode = await createPipelineNodeAction({
			pipelineId,
			nodeId: draggableNodeItem.toString(),
			nodeVersion: "v1",
			// FIXME: This should be the actual cursor position
			coords: {
				x: 250,
				y: 25,
			},
		});

		const node = props.availableNodes.find(
			(node) => node.id === pipelineNode.nodeId,
		);

		setDraggableNodeItem("");

		setNodes((nodes) => {
			return [
				...nodes,
				{
					id: pipelineNode.id,
					position: pipelineNode.coords,
					type: NodeType.Task,
					data: {
						name: `${node?.name}:${pipelineNode.nodeVersion}`,
						config: node?.config,
						inputs: pipelineNode.inputs,
						outputs: pipelineNode.outputs,
					},
				},
			];
		});
	}, [draggableNodeItem, pipelineId, props.availableNodes, setNodes]);

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

			<DndContext onDragStart={onDragStart} onDragEnd={onDragEnd}>
				<Dropzone className="h-full">
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
							nodeTypes={nodeTypes}
							fitView
							proOptions={{
								hideAttribution: true,
							}}
						>
							<Panel
								className="p-3 border rounded-md top-1 right-1 bottom-1 bg-slate-100"
								position="top-right"
							>
								<ul className="pl-5">
									{props.availableNodes.map((node) => (
										<li key={node.id} className="mb-2">
											<NodeItem id={node.id}>{node.name}</NodeItem>
										</li>
									))}
								</ul>
							</Panel>

							<Background color="#ccc" variant={BackgroundVariant.Dots} />
						</ReactFlow>

						<DragOverlay dropAnimation={null}>
							<NodeItem id="dragged-node">
								{draggableNodeItem
									? props.availableNodes.find(
											(node) => node.id === draggableNodeItem,
										)?.name
									: ""}
							</NodeItem>
						</DragOverlay>

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
				</Dropzone>
			</DndContext>
		</div>
	);
};

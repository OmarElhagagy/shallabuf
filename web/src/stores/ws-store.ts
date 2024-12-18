import { createStore } from "zustand/vanilla";

// Actions

export enum WsAction {
	Authenticate = "Authenticate",
	UpdateNodePosition = "UpdateNodePosition",
	UpdateCursorPosition = "UpdateCursorPosition",
	EnterPipelineEditor = "EnterPipelineEditor",
	LeavePipelineEditor = "LeavePipelineEditor",
}

export interface WsActionAuthenticatePayload {
	token: string;
}

export interface WsActionUpdateNodePosition {
	pipelineId: string;
	nodeId: string;
	nodePosition: Coords;
}

export interface WsActionUpdateCursorPosition {
	pipelineId: string;
	cursorPosition: Coords;
}

export interface WsActionEnterPipelineEditorPayload {
	pipelineId: string;
}

export interface WsActionLeavePipelineEditorPayload {
	pipelineId: string;
}

export type WsActionPayload =
	| WsActionAuthenticatePayload
	| WsActionUpdateNodePosition
	| WsActionUpdateCursorPosition
	| WsActionEnterPipelineEditorPayload
	| WsActionLeavePipelineEditorPayload;

export interface WsActionMessage {
	action: WsAction;
	payload: WsActionPayload;
}

// Responses

export enum WsResAction {
	AuthState = "AuthState",
	IncludePipelineEditorParticipant = "IncludePipelineEditorParticipant",
	ExcludePipelineEditorParticipant = "ExcludePipelineEditorParticipant",
	UpdateCursorPosition = "UpdateCursorPosition",
	UpdateNodePosition = "UpdateNodePosition",
}

export interface WsResAuthStatePayload {
	isAuthenticated: boolean;
}

export interface WsResIncludePipelineEditorParticipantPayload {
	pipelineId: string;
	userId: string;
	username: string;
}

export interface WsResExcludePipelineEditorParticipantPayload {
	pipelineId: string;
	userId: string;
}

export interface WsResUpdateCursorPositionPayload {
	pipelineId: string;
	userId: string;
	cursorPosition: {
		x: number;
		y: number;
	};
}

export interface WsResUpdateNodePositionPayload {
	pipelineId: string;
	nodeId: string;
	nodePosition: Coords;
}

export type WsResActionPayload =
	| WsResAuthStatePayload
	| WsResIncludePipelineEditorParticipantPayload
	| WsResExcludePipelineEditorParticipantPayload
	| WsResUpdateCursorPositionPayload
	| WsResUpdateNodePositionPayload;

export interface WsResActionMessage<T> {
	action: WsResAction;
	payload: T;
}

export const isAuthState = (
	message: WsResActionMessage<WsResActionPayload>,
): message is WsResActionMessage<WsResAuthStatePayload> => {
	return message.action === WsResAction.AuthState;
};

export const isIncludePipelineEditorParticipant = (
	message: WsResActionMessage<WsResActionPayload>,
): message is WsResActionMessage<WsResIncludePipelineEditorParticipantPayload> => {
	return message.action === WsResAction.IncludePipelineEditorParticipant;
};

export const isExcludePipelineEditorParticipant = (
	message: WsResActionMessage<WsResActionPayload>,
): message is WsResActionMessage<WsResExcludePipelineEditorParticipantPayload> => {
	return message.action === WsResAction.ExcludePipelineEditorParticipant;
};

export const isUpdateCursorPosition = (
	message: WsResActionMessage<WsResActionPayload>,
): message is WsResActionMessage<WsResUpdateCursorPositionPayload> => {
	return message.action === WsResAction.UpdateCursorPosition;
};

export const isNodePositionUpdate = (
	message: WsResActionMessage<WsResActionPayload>,
): message is WsResActionMessage<WsResUpdateNodePositionPayload> => {
	return message.action === WsResAction.UpdateNodePosition;
};

export interface Coords {
	x: number;
	y: number;
}

export interface WsStoreState {
	subscribers: Record<string, (payload: unknown) => void>;
	ws: WebSocket | null;
	delayedMessages: WsActionMessage[];
	isAuthenticated: boolean;
	pipelinesParticipants: Record<
		string,
		Record<
			string,
			{
				username: string;
				cursorPosition?: Coords;
			}
		>
	>; // { pipelineId -> { userId -> {username, cursorPosition} } }
}

export interface WsStoreActions {
	subscribeForNodeUpdates: (
		pipelineId: string,
		cb: (update: WsResUpdateNodePositionPayload) => void,
	) => () => void;
	unsubscribeForNodeUpdates: (pipelineId: string) => void;
	sendMessage: (message: WsActionMessage) => void;
	connect: (uri: string, sessionToken: string) => () => void;
	disconnect: () => void;
	authenticate: (token: string) => void;
	enterPipelineEditor: (pipelineId: string) => void;
	leavePipelineEditor: (pipelineId: string) => void;
	updateCursorPosition: (pipelineId: string, cursorPosition: Coords) => void;
	updateNodePosition: (
		pipelineId: string,
		nodeId: string,
		nodePosition: Coords,
	) => void;
	initPipelineParticipants: (
		pipelineId: string,
		participants: WsStoreState["pipelinesParticipants"][string],
	) => void;
}

export type WsStore = WsStoreState & WsStoreActions;

export const defaultInitState: WsStoreState = {
	subscribers: {},
	ws: null,
	delayedMessages: [],
	isAuthenticated: false,
	pipelinesParticipants: {},
};

export const createWsStore = (initState: WsStoreState = defaultInitState) =>
	createStore<WsStore>()((set, get) => ({
		...initState,
		subscribeForNodeUpdates: (pipelineId, cb) => {
			set((state) => ({
				subscribers: {
					...state.subscribers,
					[pipelineId]: cb as (payload: unknown) => void,
				},
			}));

			return () => {
				get().unsubscribeForNodeUpdates(pipelineId);
			};
		},
		unsubscribeForNodeUpdates: (pipelineId) => {
			set((state) => ({
				subscribers: Object.fromEntries(
					Object.entries(state.subscribers).filter(
						([key]) => key !== pipelineId,
					),
				),
			}));
		},
		sendMessage: (message) => {
			if (get().ws === null || get().ws?.readyState !== get().ws?.OPEN) {
				set({ delayedMessages: [...get().delayedMessages, message] });
				return;
			}

			console.log("Sending message", message);

			get().ws?.send(JSON.stringify(message));
		},
		authenticate: (token) => {
			const data: WsActionMessage = {
				action: WsAction.Authenticate,
				payload: {
					token,
				} as WsActionAuthenticatePayload,
			};

			get().sendMessage(data);
		},
		connect: (uri, session_token) => {
			if (get().ws === null) {
				const ws = new WebSocket(uri);
				set({ ws });

				ws.onopen = () => {
					get().authenticate(session_token);

					for (const messages of get().delayedMessages) {
						get().sendMessage(messages);
					}
				};

				ws.onmessage = (event) => {
					console.log("Received message", event.data);

					const message: WsResActionMessage<WsResActionPayload> = JSON.parse(
						event.data,
					);

					if (isAuthState(message)) {
						set({ isAuthenticated: message.payload.isAuthenticated });
					}

					if (isIncludePipelineEditorParticipant(message)) {
						const { pipelineId, userId, username } = message.payload;

						set((state) => {
							const participants =
								state.pipelinesParticipants[pipelineId] || {};

							return {
								pipelinesParticipants: {
									...state.pipelinesParticipants,
									[pipelineId]: {
										...participants,
										[userId]: {
											username,
											cursorPosition: undefined,
										},
									},
								},
							};
						});
					}

					if (isExcludePipelineEditorParticipant(message)) {
						const { pipelineId, userId } = message.payload;

						set((state) => {
							const participants =
								state.pipelinesParticipants[pipelineId] || {};

							return {
								pipelinesParticipants: {
									...state.pipelinesParticipants,
									[pipelineId]: Object.fromEntries(
										Object.entries(participants).filter(
											([key]) => key !== userId,
										),
									),
								},
							};
						});
					}

					if (isUpdateCursorPosition(message)) {
						const { pipelineId, userId, cursorPosition } = message.payload;

						set((state) => {
							const participants =
								state.pipelinesParticipants[pipelineId] ?? {};

							return {
								pipelinesParticipants: {
									...state.pipelinesParticipants,
									[pipelineId]: {
										...participants,
										[userId]: {
											...(participants[userId] ?? {}),
											cursorPosition,
										},
									},
								},
							};
						});
					}

					if (isNodePositionUpdate(message)) {
						const { pipelineId } = message.payload;

						const subscriber = get().subscribers[pipelineId];

						if (subscriber) {
							subscriber(message.payload);
						}
					}
				};
			}

			return () => {
				get().disconnect();
			};
		},
		disconnect: () => {
			if (get().ws === null) {
				return;
			}

			set({ ws: null, isAuthenticated: false });
			get().ws?.close();
		},
		enterPipelineEditor: (pipelineId) => {
			const data: WsActionMessage = {
				action: WsAction.EnterPipelineEditor,
				payload: {
					pipelineId,
				} as WsActionEnterPipelineEditorPayload,
			};

			get().sendMessage(data);
		},
		leavePipelineEditor: (pipelineId) => {
			const data: WsActionMessage = {
				action: WsAction.LeavePipelineEditor,
				payload: {
					pipelineId,
				} as WsActionLeavePipelineEditorPayload,
			};

			get().sendMessage(data);
		},
		updateCursorPosition: (pipelineId, cursorPosition) => {
			const data: WsActionMessage = {
				action: WsAction.UpdateCursorPosition,
				payload: {
					pipelineId,
					cursorPosition,
				} as WsActionUpdateCursorPosition,
			};

			get().sendMessage(data);
		},
		updateNodePosition: (pipelineId, nodeId, nodePosition) => {
			const data: WsActionMessage = {
				action: WsAction.UpdateNodePosition,
				payload: {
					pipelineId,
					nodeId,
					nodePosition,
				} as WsActionUpdateNodePosition,
			};

			get().sendMessage(data);
		},
		initPipelineParticipants: (pipelineId, participants) => {
			set((state) => ({
				pipelinesParticipants: {
					...state.pipelinesParticipants,
					[pipelineId]: participants,
				},
			}));
		},
	}));

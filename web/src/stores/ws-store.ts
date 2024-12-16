import { createStore } from "zustand/vanilla";

// Actions

export enum WsAction {
	Authenticate = "Authenticate",
	UpdateNode = "UpdateNode",
	EnterPipelineEditor = "EnterPipelineEditor",
	LeavePipelineEditor = "LeavePipelineEditor",
}

export interface WsActionAuthenticatePayload {
	token: string;
}

export interface WsActionUpdateNode {
	node_id: string;
	coords: {
		x: number;
		y: number;
	};
}

export interface WsActionEnterPipelineEditorPayload {
	pipelineId: string;
}

export interface WsActionLeavePipelineEditorPayload {
	pipelineId: string;
}

export type WsActionPayload =
	| WsActionAuthenticatePayload
	| WsActionUpdateNode
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

export type WsResActionPayload =
	| WsResAuthStatePayload
	| WsResIncludePipelineEditorParticipantPayload
	| WsResExcludePipelineEditorParticipantPayload;

export interface WsResActionMessage<T> {
	action: WsResAction;
	payload: T;
}

export const isAuthState = (
	message: WsResActionMessage<WsResActionPayload>,
): message is WsResActionMessage<WsResAuthStatePayload> => {
	return message.action === WsResAction.AuthState;
};

// export const isNodeUpdate = (
// 	message: WsActionMessage<WsActionPayload>,
// ): message is WsActionMessage<WsActionUpdateNode> => {
// 	return message.action === WsAction.UpdateNode;
// };

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

export interface WsStoreState {
	ws: WebSocket | null;
	delayedMessages: WsActionMessage[];
	isAuthenticated: boolean;
	pipelinesParticipants: Record<string, Record<string, string>>; // { pipelineId -> { userId -> username } }
}

export interface WsStoreActions {
	sendMessage: (message: WsActionMessage) => void;
	connect: (uri: string, session_token: string) => () => void;
	disconnect: () => void;
	authenticate: (token: string) => void;
	enterPipelineEditor: (pipeline_id: string) => void;
	leavePipelineEditor: (pipeline_id: string) => void;
	initPipelineParticipants: (
		pipelineId: string,
		participants: Record<string, string>,
	) => void;
}

export type WsStore = WsStoreState & WsStoreActions;

export const defaultInitState: WsStoreState = {
	ws: null,
	delayedMessages: [],
	isAuthenticated: false,
	pipelinesParticipants: {},
};

export const createWsStore = (initState: WsStoreState = defaultInitState) =>
	createStore<WsStore>()((set, get) => ({
		...initState,
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
										[userId]: username,
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
		initPipelineParticipants: (pipelineId, participants) => {
			set((state) => ({
				pipelinesParticipants: {
					...state.pipelinesParticipants,
					[pipelineId]: participants,
				},
			}));
		},
	}));

export enum WsAction {
	UpdateNode = "update_node",
	AddEditorParticipant = "add_editor_participant",
	RemoveEditorParticipant = "remove_editor_participant",
}

export interface WsActionUpdateNode {
	node_id: string;
	coords: {
		x: number;
		y: number;
	};
}

export interface WsActionAddEditorParticipant {
	pipeline_id: string;
	user_id: string;
	username: string;
}

export interface WsActionRemoveEditorParticipant {
	pipeline_id: string;
	user_id: string;
}

export type WsActionPayload =
	| WsActionUpdateNode
	| WsActionAddEditorParticipant
	| WsActionRemoveEditorParticipant;

export interface WsActionMessage<T> {
	action: WsAction;
	sender_id: string | null;
	payload: T;
}

export const isNodeUpdate = (
	message: WsActionMessage<WsActionPayload>,
): message is WsActionMessage<WsActionUpdateNode> => {
	return message.action === WsAction.UpdateNode;
};

export const isAddEditorParticipant = (
	message: WsActionMessage<WsActionPayload>,
): message is WsActionMessage<WsActionAddEditorParticipant> => {
	return message.action === WsAction.AddEditorParticipant;
};

export const isRemoveEditorParticipant = (
	message: WsActionMessage<WsActionPayload>,
): message is WsActionMessage<WsActionRemoveEditorParticipant> => {
	return message.action === WsAction.RemoveEditorParticipant;
};

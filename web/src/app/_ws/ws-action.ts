export enum WsAction {
	UpdateNode = "update_node",
}

export interface WsActionUpdateNode {
	id: string;
	coords: {
		x: number;
		y: number;
	};
	sender_id: string;
}

export interface WsActionMessage<T> {
	action: WsAction;
	payload: T;
}

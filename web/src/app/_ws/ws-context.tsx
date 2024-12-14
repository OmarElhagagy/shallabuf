"use client";
import { createContext, useContext } from "react";

export interface WsContextValue {
	ws: WebSocket | null;
}

export const WsContext = createContext<WsContextValue>({
	ws: null,
});

export const WsProvider = WsContext.Provider;
export const WsConsumer = WsContext.Consumer;

export const useWs = () => {
	return useContext(WsContext);
};

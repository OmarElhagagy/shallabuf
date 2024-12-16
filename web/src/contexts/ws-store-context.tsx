"use client";
import { createContext, useContext, useRef } from "react";
import { useStore } from "zustand";
import { type WsStore, createWsStore } from "~/stores/ws-store";

export type WsStoreApi = ReturnType<typeof createWsStore>;

export const WsStoreContext = createContext<WsStoreApi | null>(null);

export interface WsStoreProviderProps {
	children: React.ReactNode;
}

export const WsStoreProvider = ({ children }: WsStoreProviderProps) => {
	const storeRef = useRef<WsStoreApi | null>(null);

	if (!storeRef.current) {
		storeRef.current = createWsStore();
	}

	return (
		<WsStoreContext.Provider value={storeRef.current}>
			{children}
		</WsStoreContext.Provider>
	);
};

export const useWsStore = <T,>(selector: (store: WsStore) => T): T => {
	const store = useContext(WsStoreContext);

	if (!store) {
		throw new Error("useWsStore must be used within a WsStoreProvider");
	}

	return useStore(store, selector);
};

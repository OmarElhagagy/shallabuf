"use client";
import { useEffect, useMemo } from "react";
import { WsProvider } from "~/app/_ws/ws-context";

export interface ProvidersProps {
	children: React.ReactNode;
}

export const Providers = ({ children }: ProvidersProps) => {
	const ws = useMemo(
		() =>
			typeof window !== "undefined"
				? new WebSocket("ws://192.168.0.2:8000/api/v0/ws")
				: null,
		[],
	);

	useEffect(() => {
		if (!ws) {
			return;
		}

		ws.onopen = () => {
			console.log("Connected to WebSocket");
		};

		ws.onclose = () => {
			console.log("Disconnected from WebSocket");
		};

		return () => {
			ws.close();
		};
	}, [ws]);

	return <WsProvider value={{ ws }}>{children}</WsProvider>;
};

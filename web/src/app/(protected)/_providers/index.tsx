"use client";
import { useEffect } from "react";
import { useShallow } from "zustand/react/shallow";
import { useWsStore } from "~/contexts/ws-store-context";

export interface ProvidersProps {
	session_token: string;
	children: React.ReactNode;
}

export const Providers = ({ session_token, children }: ProvidersProps) => {
	const [connect] = useWsStore(useShallow((state) => [state.connect]));

	useEffect(() => {
		return connect("ws://localhost:8000/api/v0/ws", session_token);
	}, [connect, session_token]);

	return <>{children}</>;
};

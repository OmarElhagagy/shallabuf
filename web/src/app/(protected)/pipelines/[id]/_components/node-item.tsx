import { useDraggable } from "@dnd-kit/core";

export interface NodeItemProps {
	id: string;
	children: React.ReactNode;
	ariaDescribedBy?: string;
}

export const NodeItem = (props: NodeItemProps) => {
	const { attributes, listeners, setNodeRef } = useDraggable({
		id: props.id,
	});

	return (
		<div
			ref={setNodeRef}
			{...listeners}
			{...attributes}
			aria-describedby={props.ariaDescribedBy}
		>
			{props.children}
		</div>
	);
};

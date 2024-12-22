import { Handle, type Node, type NodeProps, Position } from "@xyflow/react";
import { Play as PlayIcon } from "lucide-react";
import { triggerPipelineAction } from "~/app/actions/trigger-pipeline";
import { Button } from "~/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/card";
import type { TaskNodeConfig } from "~/lib/dtos";

export type TriggerNodeProps = Node<
	{
		name: string;
		pipelineId: string;
		config: TaskNodeConfig;
	},
	"trigger"
>;

export const TriggerNode = ({
	data,
	isConnectable,
}: NodeProps<TriggerNodeProps>) => {
	return (
		<Card>
			<CardHeader>
				<CardTitle>{data.name}</CardTitle>
			</CardHeader>

			<CardContent>
				<Handle
					type="source"
					position={Position.Right}
					isConnectable={isConnectable}
				/>

				<Button
					className="flex items-center justify-center w-full"
					onClick={async () => {
						await triggerPipelineAction(data.pipelineId);
					}}
				>
					<PlayIcon />
				</Button>
			</CardContent>
		</Card>
	);
};

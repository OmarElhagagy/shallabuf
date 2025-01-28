import { Handle, type Node, type NodeProps, Position } from "@xyflow/react";
import { TriggerPipelineDialog } from "~/components/features/pipeline/trigger-pipeline-dialog";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/card";
import type { ExecStatus, TaskNodeConfig } from "~/lib/dtos";

export type TriggerNodeProps = Node<
	{
		id: string;
		name: string;
		pipelineId: string;
		config: TaskNodeConfig;
		execStatus?: ExecStatus;
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

				<TriggerPipelineDialog
					pipelineId={data.pipelineId}
					triggerId={data.id}
				/>
			</CardContent>
		</Card>
	);
};

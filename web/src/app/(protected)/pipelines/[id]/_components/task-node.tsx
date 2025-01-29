import { Label } from "@radix-ui/react-label";
import { Handle, type Node, type NodeProps, Position } from "@xyflow/react";
import {
	Check as CheckIcon,
	Image as ImageIcon,
	Text as TextIcon,
} from "lucide-react";
import { NodeInput } from "~/components/features/pipeline/node-input";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/card";
import { Separator } from "~/components/ui/separator";
import type { ExecStatus, Pipeline, TaskNodeConfig } from "~/lib/dtos";

export type TaskNodeProps = Node<
	{
		name: string;
		execStatus?: ExecStatus;
		result?: Record<string, unknown>;
		config?: TaskNodeConfig;
		triggerId?: string;
		inputs: Array<
			Pipeline["nodes"][number]["inputs"][number] & { controlled: boolean }
		>;
		outputs: Pipeline["nodes"][number]["outputs"];
	},
	"task"
>;

export const TaskNode = ({ data, isConnectable }: NodeProps<TaskNodeProps>) => {
	console.log(data.result);

	return (
		<Card>
			<CardHeader>
				<CardTitle>
					{data.name} {data.execStatus}
				</CardTitle>
			</CardHeader>

			<CardContent>
				{data.config?.inputs.map(({ key, label, input }) => {
					const inputHandle = data.inputs.find((input) => input.key === key);

					const willBeComputed =
						inputHandle?.controlled || Boolean(data.triggerId);

					return (
						<div key={key} className="relative [&:not(:first-child)]:mt-2">
							<Handle
								key={key}
								id={data.inputs.find((input) => input.key === key)?.id}
								type="target"
								position={Position.Left}
								isConnectable={isConnectable}
								style={{
									left: "-1.5rem",
									transform: "translate(-50%, 8px)",
								}}
							/>

							<NodeInput
								label={label.en}
								input={input}
								willBeComputed={willBeComputed}
							/>
						</div>
					);
				})}

				<Separator className="my-4" />

				{data.config?.outputs.map(({ key, label, output }) => (
					<div
						key={key}
						className="relative flex items-center [&:not(:first-child)]:mt-2"
					>
						<Label>{label.en}</Label>

						{JSON.stringify(data.result)}

						{output === "text" && <TextIcon className="ml-auto" />}
						{output === "status" && <CheckIcon className="ml-auto" />}
						{output === "binary" && <ImageIcon className="ml-auto" />}

						<Handle
							id={data.outputs.find((output) => output.key === key)?.id}
							type="source"
							style={{
								right: "-1.5rem",
								transform: "translate(50%, -50%)",
							}}
							position={Position.Right}
							isConnectable={isConnectable}
						/>
					</div>
				))}
			</CardContent>
		</Card>
	);
};

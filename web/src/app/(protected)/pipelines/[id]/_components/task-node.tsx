import { Label } from "@radix-ui/react-label";
import { Handle, type Node, type NodeProps, Position } from "@xyflow/react";
import {
	Check as CheckIcon,
	Image as ImageIcon,
	Text as TextIcon,
} from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/card";
import { Input } from "~/components/ui/input";
import { Separator } from "~/components/ui/separator";
import {
	type TaskNodeConfig,
	isTaskNodeConfigV0InputBinary,
	isTaskNodeConfigV0InputSelect,
	isTaskNodeConfigV0InputText,
} from "~/lib/dtos";
import { SelectInput } from "./select-input";

export type TaskNodeProps = Node<
	{
		name: string;
		config: TaskNodeConfig;
	},
	"task"
>;

export const TaskNode = ({ data, isConnectable }: NodeProps<TaskNodeProps>) => {
	return (
		<Card>
			<CardHeader>
				<CardTitle>{data.name}</CardTitle>
			</CardHeader>

			<CardContent>
				{data.config.inputs.map(({ name, label, input }) => {
					return (
						<div key={name} className="relative [&:not(:first-child)]:mt-2">
							<Handle
								key={name}
								id={name}
								type="target"
								position={Position.Left}
								isConnectable={isConnectable}
								style={{
									left: "-1.5rem",
									transform: "translate(-50%, 8px)",
								}}
							/>

							<Label>{label.en}</Label>

							{isTaskNodeConfigV0InputText(input) && (
								<Input defaultValue={input.text.default} />
							)}

							{isTaskNodeConfigV0InputSelect(input) && (
								<SelectInput {...input.select} />
							)}

							{isTaskNodeConfigV0InputBinary(input) && (
								<Input type="file" accept="image/*" />
							)}
						</div>
					);
				})}

				<Separator className="my-4" />

				{data.config.outputs.map((output, index) => (
					// biome-ignore lint/suspicious/noArrayIndexKey: This is a unique identifier
					<div key={index} className="relative [&:not(:first-child)]:mt-2">
						{output === "text" && <TextIcon className="ml-auto" />}
						{output === "status" && <CheckIcon className="ml-auto" />}
						{output === "binary" && <ImageIcon className="ml-auto" />}

						<Handle
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

import { SelectInput } from "~/app/(protected)/pipelines/[id]/_components/select-input";
import { Input } from "~/components/ui/input";
import { Label } from "~/components/ui/label";
import {
	type TaskNodeConfigV0Input,
	isTaskNodeConfigV0InputBinary,
	isTaskNodeConfigV0InputSelect,
	isTaskNodeConfigV0InputText,
} from "~/lib/dtos";

export interface NodeInputProps {
	label: string;
	input: TaskNodeConfigV0Input;
	willBeComputed?: boolean;
	value?: string;
	onChange?: (...event: unknown[]) => void;
}

export const NodeInput = ({
	label,
	input,
	willBeComputed,
	value,
	onChange,
}: NodeInputProps) => {
	return (
		<>
			<Label>{label}</Label>

			{isTaskNodeConfigV0InputText(input) && (
				<Input
					value={value}
					onChange={onChange}
					disabled={willBeComputed}
					{...(!value && { defaultValue: input.text.default })}
					placeholder={willBeComputed ? "Will be computed" : ""}
				/>
			)}

			{isTaskNodeConfigV0InputSelect(input) && (
				<SelectInput
					value={value}
					onChange={onChange}
					options={input.select.options}
					{...(!value && { defaultValue: input.select.default })}
				/>
			)}

			{isTaskNodeConfigV0InputBinary(input) && (
				<Input value={value} onChange={onChange} type="file" accept="image/*" />
			)}
		</>
	);
};

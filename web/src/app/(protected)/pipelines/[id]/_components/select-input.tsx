import { useState } from "react";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "~/components/ui/select";
import type { TaskNodeConfigV0InputSelect } from "~/lib/dtos";

export const SelectInput = (props: TaskNodeConfigV0InputSelect["select"]) => {
	const [value, setValue] = useState(props.default);
	const displayValue = props.options.find((option) => option.value === value)
		?.label.en;

	return (
		<Select defaultValue={props.default} value={value} onValueChange={setValue}>
			<SelectTrigger>
				<SelectValue>{displayValue}</SelectValue>
			</SelectTrigger>

			<SelectContent>
				{props.options.map((option) => (
					<SelectItem key={option.value} value={option.value}>
						{option.label.en}
					</SelectItem>
				))}
			</SelectContent>
		</Select>
	);
};

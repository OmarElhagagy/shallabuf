import { useCallback, useState } from "react";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "~/components/ui/select";
import type { TaskNodeConfigV0InputSelect } from "~/lib/dtos";

export interface SelectInputProps {
  defaultValue?: string;
  options: TaskNodeConfigV0InputSelect["select"]["options"];
  value?: string;
  onChange?: (value: string) => void;
}

export const SelectInput = ({
  defaultValue,
  options,
  value,
  onChange,
}: SelectInputProps) => {
  const [currentValue, setCurrentValue] = useState(value ?? defaultValue);
  const displayValue = options.find((option) => option.value === value)?.label
    .en;

  const onValueChangeHandler = useCallback(
    (nextValue: string) => {
      onChange?.(nextValue);
      setCurrentValue(nextValue);
    },
    [onChange],
  );

  return (
    <Select
      defaultValue={defaultValue}
      value={currentValue}
      onValueChange={onValueChangeHandler}
    >
      <SelectTrigger>
        <SelectValue>{displayValue}</SelectValue>
      </SelectTrigger>

      <SelectContent>
        {options.map((option) => (
          <SelectItem key={option.value} value={option.value}>
            {option.label.en}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
};

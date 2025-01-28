import type { PipelineExec } from "./dtos";

export const isPipelineExec = (data: PipelineExec): data is PipelineExec => {
	return "pipelineId" in data;
};

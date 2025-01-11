"use client";
import { zodResolver } from "@hookform/resolvers/zod";
import { Loader } from "lucide-react";
import { memo, useActionState, useCallback, useState } from "react";
import { useForm } from "react-hook-form";
import type { z } from "zod";
import { createPipelineAction } from "~/actions/create-pipeline";
import { Button } from "~/components/ui/button";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
	DialogTrigger,
} from "~/components/ui/dialog";
import {
	Form,
	FormControl,
	FormField,
	FormFieldMessage,
	FormItem,
	FormLabel,
} from "~/components/ui/form";
import { Input } from "~/components/ui/input";
import { Textarea } from "~/components/ui/textarea";
import { createPipelineSchema } from "~/lib/schemas";

export interface CreatePipelineDialogProps {
	teamId: string;
}

export const CreatePipelineDialog = memo(
	({ teamId }: CreatePipelineDialogProps) => {
		const form = useForm<z.infer<typeof createPipelineSchema>>({
			resolver: zodResolver(createPipelineSchema),
			defaultValues: {
				teamId,
				name: "",
				description: "",
			},
		});

		const [, formAction] = useActionState(createPipelineAction, {
			errors: {
				teamId: undefined,
				name: undefined,
				description: undefined,
			},
		});

		const [open, setOpen] = useState(false);

		const submit = useCallback(
			(formData: FormData) => {
				formAction(formData);
				setOpen(false);
			},
			[formAction],
		);

		return (
			<Dialog open={open} onOpenChange={setOpen}>
				<DialogTrigger asChild>
					<Button variant="default">Create</Button>
				</DialogTrigger>
				<DialogContent>
					<DialogHeader>
						<DialogTitle>Create new pipeline</DialogTitle>
						<DialogDescription>
							This action will create a new pipeline. Please fill out the form
							below.
						</DialogDescription>
					</DialogHeader>

					<Form {...form}>
						<form className="flex flex-col items-center gap-4" action={submit}>
							<FormField
								name="teamId"
								control={form.control}
								render={({ field }) => <input type="hidden" {...field} />}
							/>

							<FormField
								name="name"
								control={form.control}
								render={({ field }) => (
									<FormItem className="min-w-full">
										<FormLabel>Name</FormLabel>

										<FormControl>
											<Input placeholder="Pipeline name" {...field} />
										</FormControl>

										<FormFieldMessage />
									</FormItem>
								)}
							/>

							<FormField
								name="description"
								control={form.control}
								render={({ field }) => (
									<FormItem className="min-w-full">
										<FormLabel>Description</FormLabel>

										<FormControl>
											<Textarea
												placeholder="Pipeline description"
												className="resize-none"
												{...field}
											/>
										</FormControl>

										<FormFieldMessage />
									</FormItem>
								)}
							/>

							<Button className="ml-auto" type="submit">
								{form.formState.isSubmitting ? <Loader /> : null}
								Create Pipeline
							</Button>
						</form>
					</Form>
				</DialogContent>
			</Dialog>
		);
	},
);

CreatePipelineDialog.displayName = "CreatePipelineDialog";

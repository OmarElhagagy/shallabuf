import { CreatePipelineDialog } from "~/components/features/pipeline/create-pipeline-dialog";
import { env } from "~/env";
import { getSessionToken } from "~/lib/auth";

interface Pipeline {
	id: string;
	name: string;
	description: string;
}

interface Team {
	id: string;
	name: string;
}

export default async function Home() {
	const sessionToken = await getSessionToken();

	const pipelines_req = fetch(`${env.API_URL}/pipelines`, {
		headers: {
			Authorization: `Bearer ${sessionToken}`,
		},
	});

	const teams_req = fetch(`${env.API_URL}/teams`, {
		headers: {
			Authorization: `Bearer ${sessionToken}`,
		},
	});

	const [pipelines_res, teams_res] = await Promise.all([
		pipelines_req,
		teams_req,
	]);

	const [pipelines, teams]: [Pipeline[], Team[]] = await Promise.all([
		pipelines_res.json(),
		teams_res.json(),
	]);

	return (
		<div className="grid grid-rows-[20px_1fr_20px] items-center justify-items-center min-h-screen p-8 pb-20 gap-16 sm:p-20 font-[family-name:var(--font-geist-sans)]">
			<header className="flex items-center justify-center gap-4">
				<h1 className="text-3xl font-bold text-center">Pipelines</h1>
				<CreatePipelineDialog teamId={teams[0].id} />
			</header>

			<ul className="w-full max-w-2xl">
				{pipelines.map((pipeline) => (
					<li
						key={pipeline.id}
						className="border border-gray-300 rounded p-4 mt-4 shadow-md hover:shadow-lg transition-shadow duration-300"
					>
						<h2 className="text-xl font-bold mb-2">{pipeline.name}</h2>
						<p className="text-gray-700">{pipeline.description}</p>
						<a href={`/pipelines/${pipeline.id}`}>Edit</a>
					</li>
				))}
			</ul>
		</div>
	);
}

interface Pipeline {
	id: number;
	name: string;
	description: string;
}

export default async function Home() {
	const data = await fetch("http://localhost:8000/api/v0/pipelines");
	const pipelines: Pipeline[] = await data.json();

	return (
		<div className="grid grid-rows-[20px_1fr_20px] items-center justify-items-center min-h-screen p-8 pb-20 gap-16 sm:p-20 font-[family-name:var(--font-geist-sans)]">
			<h1 className="text-3xl font-bold text-center">Pipelines</h1>

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

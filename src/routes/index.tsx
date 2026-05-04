import { createFileRoute } from "@tanstack/react-router";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

export const Route = createFileRoute("/")({
	component: Home,
});

function Home() {
	const handleClick = async () => {
		const file = await open({
			multiple: false,
			directory: false,
		});

		if (!file) return;

		await invoke("dump_metadata", { path: file });
	};

	return (
		<h1 className="text-4xl">
			Hello Tauri + Tanstack!
			<button type="button" onClick={handleClick}>あ</button>
		</h1>
	);
}

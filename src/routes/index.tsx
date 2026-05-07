import { createFileRoute } from "@tanstack/react-router";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button";

export const Route = createFileRoute("/")({
	component: Home,
});

function Home() {
	const handleClick = async () => {
		const file = await open({
			multiple: false,
			directory: true,
		});

		if (!file) return;

		await invoke("scan_library", { baseDir: file });
	};

	return (
		<h1 className="text-4xl">
			Hello Tauri + Tanstack!
			<Button variant="outline" onClick={handleClick}>
				あ
			</Button>
		</h1>
	);
}

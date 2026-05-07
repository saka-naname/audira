import { createFileRoute, Link, useNavigate } from "@tanstack/react-router";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useEffect } from "react";

export const Route = createFileRoute("/")({
  component: Home,
});

function Home() {
  const navigate = useNavigate({ from: "/" });

  const handleClick = async () => {
    const file = await open({
      multiple: false,
      directory: true,
    });

    if (!file) return;

    await invoke("scan_library", { baseDir: file });
  };

  useEffect(() => {
    navigate({ to: "/library/" });
  });

  return null;
}

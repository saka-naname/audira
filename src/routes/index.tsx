import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useEffect } from "react";

export const Route = createFileRoute("/")({
  component: Home,
});

function Home() {
  const navigate = useNavigate({ from: "/" });

  useEffect(() => {
    navigate({ to: "/library/" });
  });

  return null;
}

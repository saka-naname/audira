import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/library/")({
  component: RouteComponent,
});

function RouteComponent() {
  return <div className="h-full w-full bg-accent">Hello "/library/"!</div>;
}

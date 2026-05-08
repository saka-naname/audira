import { createFileRoute } from "@tanstack/react-router";
import SongsList from "@/features/songs/components/songs-list";

export const Route = createFileRoute("/library/")({
  component: RouteComponent,
});

function RouteComponent() {
  return <SongsList />;
}

import { createFileRoute } from "@tanstack/react-router";
import SongsList from "@/features/songs/components/songs-list";
import type { SongSort } from "@/features/songs/types";

const defaultSort: SongSort = {
  sortBy: "title",
  sortOrder: "asc",
};

export const Route = createFileRoute("/library/")({
  validateSearch: (search: Record<string, unknown>): SongSort => ({
    sortBy: isSongSortBy(search.sortBy) ? search.sortBy : defaultSort.sortBy,
    sortOrder: isSongSortOrder(search.sortOrder)
      ? search.sortOrder
      : defaultSort.sortOrder,
  }),
  component: RouteComponent,
});

function RouteComponent() {
  const sort = Route.useSearch();
  const navigate = Route.useNavigate();

  const changeSortBy = (sortBy: SongSort["sortBy"]) => {
    navigate({
      search: {
        ...sort,
        sortBy,
      },
    });
  };

  const toggleSortOrder = () => {
    navigate({
      search: {
        ...sort,
        sortOrder: sort.sortOrder === "asc" ? "desc" : "asc",
      },
    });
  };

  return (
    <SongsList
      onSortByChange={changeSortBy}
      onSortOrderToggle={toggleSortOrder}
      sort={sort}
    />
  );
}

function isSongSortBy(value: unknown): value is SongSort["sortBy"] {
  return value === "title" || value === "artist";
}

function isSongSortOrder(value: unknown): value is SongSort["sortOrder"] {
  return value === "asc" || value === "desc";
}

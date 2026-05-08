import { createFileRoute } from "@tanstack/react-router";
import AlbumsList from "@/features/albums/components/albums-list";
import type { AlbumSort } from "@/features/albums/types";

const defaultSort: AlbumSort = {
  sortBy: "title",
  sortOrder: "asc",
};

export const Route = createFileRoute("/library/albums")({
  validateSearch: (search: Record<string, unknown>): AlbumSort => ({
    sortBy: isAlbumSortBy(search.sortBy) ? search.sortBy : defaultSort.sortBy,
    sortOrder: isAlbumSortOrder(search.sortOrder)
      ? search.sortOrder
      : defaultSort.sortOrder,
  }),
  component: RouteComponent,
});

function RouteComponent() {
  const sort = Route.useSearch();
  const navigate = Route.useNavigate();

  const changeSortBy = (sortBy: AlbumSort["sortBy"]) => {
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
    <AlbumsList
      onSortByChange={changeSortBy}
      onSortOrderToggle={toggleSortOrder}
      sort={sort}
    />
  );
}

function isAlbumSortBy(value: unknown): value is AlbumSort["sortBy"] {
  return value === "title" || value === "artist";
}

function isAlbumSortOrder(value: unknown): value is AlbumSort["sortOrder"] {
  return value === "asc" || value === "desc";
}

import { createFileRoute, Outlet } from "@tanstack/react-router";
import { SidebarProvider } from "@/components/ui/sidebar";
import LibrarySidebar from "@/features/library/components/library-sidebar";

export const Route = createFileRoute("/library")({
  component: LibraryLayoutComponent,
});

function LibraryLayoutComponent() {
  return (
    <SidebarProvider>
      <LibrarySidebar />

      <main className="grow">
        <Outlet />
      </main>
    </SidebarProvider>
  );
}

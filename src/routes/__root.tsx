import {
  createRootRouteWithContext,
  HeadContent,
  Outlet,
  Scripts,
} from "@tanstack/react-router";
import type { ReactNode } from "react";
import { TooltipProvider } from "@/components/ui/tooltip";
import type { RouterContext } from "@/lib/query-client";

export const Route = createRootRouteWithContext<RouterContext>()({
  head: () => ({}),
  component: RootComponent,
});

function RootComponent() {
  return (
    <RootDocument>
      <Outlet />
    </RootDocument>
  );
}

function RootDocument({ children }: Readonly<{ children: ReactNode }>) {
  return (
    <html lang="ja">
      <head>
        <HeadContent />
      </head>
      <body>
        <TooltipProvider>
          {children}
          <Scripts />
        </TooltipProvider>
      </body>
    </html>
  );
}

import React from "react";
import ReactDOM from "react-dom/client";
import "./styles.css";
import { QueryClientProvider } from "@tanstack/react-query";
import { createRouter, RouterProvider } from "@tanstack/react-router";
import { queryClient } from "@/lib/query-client";
import { routeTree } from "./routeTree.gen";

const router = createRouter({
  routeTree,
  scrollRestoration: true,
  context: {
    queryClient,
  },
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  </React.StrictMode>,
);

import { createRootRoute, HeadContent, Outlet, Scripts } from "@tanstack/react-router";
import { ReactNode } from "react";

export const Route = createRootRoute({
    head: () => ({

    }),
    component: RootComponent
})

function RootComponent() {
    return (<RootDocument><Outlet /></RootDocument>)
}

function RootDocument({children}: Readonly<{children: ReactNode}>) {
    return (
        <html>
            <head>
                <HeadContent />
            </head>
            <body>
                { children }
                <Scripts />
            </body>
        </html>
    )
}
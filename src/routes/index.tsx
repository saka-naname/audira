import { createFileRoute } from "@tanstack/react-router"

export const Route = createFileRoute("/")({
    component: Home
})

function Home() {
    return (
        <>
            <h1 className="text-4xl">Hello Tauri + Tanstack!</h1>
        </>
    )
}
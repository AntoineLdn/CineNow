import { Outlet } from "react-router";
import NavRail from "./NavRail";

function AppLayout() {
    return (
        <div className="flex h-screen bg-base text-ink">
            <NavRail />
            <main className="flex-1 overflow-y-auto">
                <Outlet />
            </main>
        </div>
    );
}

export default AppLayout;

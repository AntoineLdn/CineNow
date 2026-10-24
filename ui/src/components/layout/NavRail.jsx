import { NavLink, Link } from "react-router";
import { Play, LayoutGrid, Bookmark, User } from "lucide-react";
import { useAuth } from "../../context/AuthContext";

const NAV_ITEMS = [
    { to: "/", label: "Ce soir", Icon: Play, end: true },
    { to: "/catalogue", label: "Catalogue", Icon: LayoutGrid },
    { to: "/liste", label: "Ma liste", Icon: Bookmark },
    { to: "/profil", label: "Profil", Icon: User },
];

function itemClass({ isActive }) {
    return `w-16 min-h-16 flex flex-col items-center justify-center gap-1.5 rounded-surface transition-colors ${
        isActive ? "bg-raised text-ink" : "text-muted hover:text-ink hover:bg-surface"
    }`;
}

function NavRail() {
    const { username } = useAuth();
    const initial = username?.trim()?.charAt(0).toUpperCase() || "?";

    return (
        <nav
            aria-label="Navigation principale"
            className="w-rail shrink-0 bg-rail border-r border-edge flex flex-col items-center py-6"
        >
            <span role="img" aria-label="CinéNow" className="font-display text-2xl text-gold">
                CN
            </span>

            <ul className="flex flex-col items-center gap-2 mt-8">
                {NAV_ITEMS.map(({ to, label, Icon, end }) => (
                    <li key={to}>
                        <NavLink to={to} end={end} className={itemClass}>
                            <Icon size={20} strokeWidth={1.5} aria-hidden="true" />
                            <span className="text-[11px] font-semibold">{label}</span>
                        </NavLink>
                    </li>
                ))}
            </ul>

            <Link
                to="/profil"
                aria-label={username ? `Profil de ${username}` : "Profil"}
                className="mt-auto w-11 h-11 rounded-pill border border-edge flex items-center justify-center text-sm font-bold text-ink transition-colors hover:border-gold"
            >
                {initial}
            </Link>
        </nav>
    );
}

export default NavRail;

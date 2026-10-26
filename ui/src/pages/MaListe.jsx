import { useState } from "react";
import { Link } from "react-router";
import { Heart, Eye } from "lucide-react";
import { useUserData } from "../context/UserDataContext";
import { useMoviesByIds } from "../hooks/useMoviesByIds";
import PosterCard from "../components/movie/PosterCard";

const TABS = [
    {
        id: "like",
        label: "Aimés",
        Icon: Heart,
        subtitle: "Les films qui t'ont plu. Leurs genres pèsent dans tes recommandations.",
        empty: "Tu n'as encore aimé aucun film.",
    },
    {
        id: "watched",
        label: "Vus",
        Icon: Eye,
        subtitle: "Les films que tu as déjà vus.",
        empty: "Tu n'as encore marqué aucun film comme vu.",
    },
];

function MaListe() {
    const { likedIds, watchedIds } = useUserData();
    const [activeTab, setActiveTab] = useState("like");

    const tab = TABS.find((t) => t.id === activeTab);
    const ids = activeTab === "like" ? likedIds : watchedIds;
    const { movies, status } = useMoviesByIds(ids);

    const counts = { like: likedIds.length, watched: watchedIds.length };

    return (
        <div className="p-12 max-w-7xl">
            <header className="flex flex-col lg:flex-row lg:items-start justify-between gap-6">
                <div>
                    <h1 className="text-greeting leading-none">Ma liste</h1>
                    <p className="text-muted mt-3">{tab.subtitle}</p>
                </div>

                <div
                    role="tablist"
                    aria-label="Filtrer ma liste"
                    className="flex gap-1 p-1 rounded-pill bg-surface shrink-0"
                >
                    {TABS.map(({ id, label, Icon }) => {
                        const isActive = id === activeTab;
                        return (
                            <button
                                key={id}
                                role="tab"
                                aria-selected={isActive}
                                onClick={() => setActiveTab(id)}
                                className={`h-pill px-5 rounded-pill font-semibold transition-colors ${
                                    isActive ? "bg-gold text-on-gold" : "text-muted hover:text-ink"
                                }`}
                            >
                                <span className="flex items-center gap-2">
                                    <Icon size={18} strokeWidth={1.5} aria-hidden="true" />
                                    {label} ({counts[id]})
                                </span>
                            </button>
                        );
                    })}
                </div>
            </header>

            <div className="mt-10">
                {ids.length === 0 ? (
                    <div className="bg-surface rounded-surface p-10 text-center">
                        <p className="text-ink">{tab.empty}</p>
                        <Link
                            to="/"
                            className="inline-flex items-center h-pill px-6 rounded-pill bg-gold text-on-gold font-semibold mt-6 transition-opacity hover:opacity-90"
                        >
                            Trouver un film pour ce soir
                        </Link>
                    </div>
                ) : status === "loading" ? (
                    <p className="text-muted">Chargement de ta liste...</p>
                ) : status === "error" ? (
                    <p role="alert" className="text-ink">
                        Ta liste n'a pas pu être chargée.
                    </p>
                ) : (
                    <div className="grid grid-cols-2 md:grid-cols-4 xl:grid-cols-6 gap-5">
                        {movies.map((movie) => (
                            <PosterCard key={movie.id} movie={movie} action={activeTab} />
                        ))}
                    </div>
                )}
            </div>
        </div>
    );
}

export default MaListe;

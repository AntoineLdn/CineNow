import { useEffect, useMemo, useState } from "react";
import { Search } from "lucide-react";
import { getMovies } from "../api/client";
import PosterCard from "../components/movie/PosterCard";

const TABS = [
    { id: "recent", label: "Sorties récentes" },
    { id: "upcoming", label: "À venir" },
];

const RECENT_LIMIT = 20;
const UPCOMING_LIMIT = 10;

const normalizeText = (text) => text.toLowerCase().normalize("NFD").replace(/[̀-ͯ]/g, "");

function Catalogue() {
    const [moviesByGenre, setMoviesByGenre] = useState({});
    const [status, setStatus] = useState("loading"); // loading | done | error
    const [activeTab, setActiveTab] = useState("recent");
    const [searchTerm, setSearchTerm] = useState("");

    useEffect(() => {
        let cancelled = false;

        getMovies()
            .then((data) => {
                if (cancelled) return;
                setMoviesByGenre(data);
                setStatus("done");
            })
            .catch(() => {
                if (!cancelled) setStatus("error");
            });

        return () => {
            cancelled = true;
        };
    }, []);

    // Un même film apparaît dans plusieurs genres : on le garde une seule fois.
    const allMovies = useMemo(() => {
        const seen = new Set();
        return Object.values(moviesByGenre)
            .flat()
            .filter((movie) => {
                if (seen.has(movie.id)) return false;
                seen.add(movie.id);
                return true;
            });
    }, [moviesByGenre]);

    const today = new Date().toISOString().slice(0, 10);

    const recentMovies = useMemo(
        () =>
            allMovies
                .filter((movie) => movie.release_date && movie.release_date <= today)
                .sort((a, b) => b.release_date.localeCompare(a.release_date))
                .slice(0, RECENT_LIMIT),
        [allMovies, today]
    );

    const upcomingMovies = useMemo(
        () =>
            allMovies
                .filter((movie) => movie.release_date && movie.release_date > today)
                .sort((a, b) => a.release_date.localeCompare(b.release_date))
                .slice(0, UPCOMING_LIMIT),
        [allMovies, today]
    );

    // La recherche porte sur tout le catalogue, pas seulement sur l'onglet ouvert.
    const searchResults = useMemo(() => {
        const needle = normalizeText(searchTerm.trim());
        if (!needle) return null;
        return allMovies.filter((movie) => normalizeText(movie.title).includes(needle));
    }, [searchTerm, allMovies]);

    const movies = searchResults ?? (activeTab === "recent" ? recentMovies : upcomingMovies);
    const heading = searchResults
        ? `Résultats pour « ${searchTerm.trim()} »`
        : TABS.find((tab) => tab.id === activeTab).label;

    return (
        <div className="p-12 max-w-7xl">
            <header className="flex flex-col lg:flex-row lg:items-end justify-between gap-6">
                <div className="min-w-0">
                    <h1 className="text-greeting leading-none truncate">{heading}</h1>

                    <div
                        role="tablist"
                        aria-label="Filtrer le catalogue"
                        className="flex gap-1 p-1 rounded-pill bg-surface mt-5 w-fit"
                    >
                        {TABS.map(({ id, label }) => {
                            const isActive = !searchResults && id === activeTab;
                            return (
                                <button
                                    key={id}
                                    role="tab"
                                    aria-selected={isActive}
                                    onClick={() => {
                                        setSearchTerm("");
                                        setActiveTab(id);
                                    }}
                                    className={`h-pill px-5 rounded-pill font-semibold transition-colors ${
                                        isActive
                                            ? "bg-gold text-on-gold"
                                            : "text-muted hover:text-ink"
                                    }`}
                                >
                                    {label}
                                </button>
                            );
                        })}
                    </div>
                </div>

                <div className="relative w-full lg:w-96 shrink-0">
                    <Search
                        size={18}
                        strokeWidth={1.5}
                        aria-hidden="true"
                        className="absolute left-5 top-1/2 -translate-y-1/2 text-muted pointer-events-none"
                    />
                    <input
                        type="search"
                        value={searchTerm}
                        onChange={(e) => setSearchTerm(e.target.value)}
                        placeholder="Rechercher un film..."
                        aria-label="Rechercher un film"
                        className="w-full h-pill pl-12 pr-5 rounded-pill bg-surface border border-edge text-ink placeholder:text-muted transition-colors focus:border-gold"
                    />
                </div>
            </header>

            <div className="mt-10">
                {status === "loading" ? (
                    <p className="text-muted">Chargement du catalogue...</p>
                ) : status === "error" ? (
                    <p role="alert" className="text-ink">
                        Le catalogue n'a pas pu être chargé.
                    </p>
                ) : movies.length === 0 ? (
                    <p className="text-muted">
                        {searchResults ? "Aucun film ne porte ce titre." : "Aucun film à afficher."}
                    </p>
                ) : (
                    <div className="grid grid-cols-2 md:grid-cols-4 xl:grid-cols-6 gap-5">
                        {movies.map((movie) => (
                            <PosterCard key={movie.id} movie={movie} />
                        ))}
                    </div>
                )}
            </div>
        </div>
    );
}

export default Catalogue;

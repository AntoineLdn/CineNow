import { useEffect, useState } from "react";
import { Link, useLocation, useNavigate, useParams } from "react-router";
import { ChevronLeft, Heart, Eye, Star, Film } from "lucide-react";
import { getMovieDetails } from "../api/client";
import { posterUrl } from "../utils/poster";
import { formatRating, formatDuration, initials, isUpcoming } from "../utils/format";
import { genreLabel } from "../data/genres";
import { useUserData } from "../context/UserDataContext";
import ToggleButton from "../components/ui/ToggleButton";

function BackLink() {
    const navigate = useNavigate();
    const location = useLocation();

    // Arrivé directement sur la fiche (lien partagé, rechargement) : il n'y a
    // rien derrière dans l'historique, on renvoie donc à l'accueil.
    if (location.key === "default") {
        return (
            <Link to="/" className="inline-flex items-center gap-2 text-gold font-semibold">
                <ChevronLeft size={20} strokeWidth={1.5} aria-hidden="true" />
                Retour à ta sélection
            </Link>
        );
    }

    return (
        <button
            onClick={() => navigate(-1)}
            className="inline-flex items-center gap-2 text-gold font-semibold"
        >
            <ChevronLeft size={20} strokeWidth={1.5} aria-hidden="true" />
            Retour à ta sélection
        </button>
    );
}

function CastMember({ name, role }) {
    return (
        <li className="flex items-center gap-3">
            <span
                aria-hidden="true"
                className="w-11 h-11 rounded-pill bg-raised text-muted text-xs font-semibold flex items-center justify-center shrink-0"
            >
                {initials(name)}
            </span>
            <span className="min-w-0">
                <span className="block text-ink font-semibold truncate">{name}</span>
                {role && <span className="block text-sm text-muted">{role}</span>}
            </span>
        </li>
    );
}

function FilmPage() {
    const { id } = useParams();
    const { isLiked, isWatched, toggleLike, toggleWatched } = useUserData();
    const [movie, setMovie] = useState(null);
    const [status, setStatus] = useState("loading"); // loading | done | error

    useEffect(() => {
        let cancelled = false;
        setStatus("loading");
        setMovie(null);

        getMovieDetails(id)
            .then((data) => {
                if (cancelled) return;
                setMovie(data);
                setStatus("done");
            })
            .catch(() => {
                if (!cancelled) setStatus("error");
            });

        return () => {
            cancelled = true;
        };
    }, [id]);

    if (status === "loading") {
        return (
            <div className="p-12">
                <BackLink />
                <p className="text-muted mt-10">Chargement de la fiche...</p>
            </div>
        );
    }

    if (status === "error") {
        return (
            <div className="p-12">
                <BackLink />
                <p role="alert" className="text-ink mt-10">
                    Cette fiche n'a pas pu être chargée.
                </p>
            </div>
        );
    }

    const poster = posterUrl(movie.poster_path);
    const year = movie.release_date?.slice(0, 4);
    const duration = formatDuration(movie.duration);
    const rating = formatRating(movie.rating);
    const showsNoRating = !rating && !isUpcoming(movie.release_date);

    return (
        <div className="p-12 max-w-7xl">
            <BackLink />

            <div className="flex flex-col lg:flex-row gap-12 mt-10">
                <div className="w-full lg:w-105 shrink-0 aspect-2/3 rounded-surface overflow-hidden bg-surface">
                    {poster ? (
                        <img src={poster} alt="" className="w-full h-full object-cover" />
                    ) : (
                        <div className="w-full h-full flex items-center justify-center text-edge">
                            <Film size={48} strokeWidth={1.5} aria-hidden="true" />
                        </div>
                    )}
                </div>

                <div className="flex-1 min-w-0">
                    <h1 className="text-greeting leading-none">{movie.title}</h1>

                    <p className="flex flex-wrap items-center gap-x-5 gap-y-1 text-muted mt-4">
                        {year && <span>{year}</span>}
                        {duration && <span>{duration}</span>}
                        {movie.director && <span>{movie.director}</span>}
                        {rating && (
                            <span className="flex items-center gap-1.5 text-gold font-semibold">
                                <Star
                                    size={15}
                                    strokeWidth={1.5}
                                    fill="currentColor"
                                    aria-hidden="true"
                                />
                                {rating} sur TMDb
                            </span>
                        )}
                        {showsNoRating && <span>Aucune note sur TMDb</span>}
                    </p>

                    {movie.genres?.length > 0 && (
                        <ul className="flex flex-wrap gap-2 mt-5">
                            {movie.genres.map((genre) => (
                                <li
                                    key={genre}
                                    className="px-4 py-1.5 rounded-pill bg-raised text-sm text-ink"
                                >
                                    {genreLabel(genre)}
                                </li>
                            ))}
                        </ul>
                    )}

                    <div className="flex flex-wrap gap-3 mt-7">
                        <ToggleButton
                            active={isLiked(movie.id)}
                            Icon={Heart}
                            fillWhenActive
                            activeLabel="Tu aimes"
                            idleLabel="J'aime"
                            onClick={() => toggleLike(movie.id)}
                        />
                        <ToggleButton
                            active={isWatched(movie.id)}
                            Icon={Eye}
                            activeLabel="Déjà vu"
                            idleLabel="Marquer comme vu"
                            onClick={() => toggleWatched(movie.id)}
                        />
                    </div>

                    {movie.overview && (
                        <p className="text-prose text-lg leading-relaxed mt-9">{movie.overview}</p>
                    )}

                    {(movie.actors?.length > 0 || movie.director) && (
                        <section className="mt-10">
                            <h2 className="font-sans text-lg font-bold text-ink">Distribution</h2>
                            <ul className="flex flex-wrap gap-x-10 gap-y-4 mt-4">
                                {movie.actors?.map((actor) => (
                                    <CastMember key={actor} name={actor} />
                                ))}
                                {movie.director && (
                                    <CastMember name={movie.director} role="Réalisation" />
                                )}
                            </ul>
                        </section>
                    )}
                </div>
            </div>
        </div>
    );
}

export default FilmPage;

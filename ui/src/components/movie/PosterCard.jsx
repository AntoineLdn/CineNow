import { Link } from "react-router";
import { Heart, Eye, Film } from "lucide-react";
import { posterUrl } from "../../utils/poster";
import { formatRating, isUpcoming } from "../../utils/format";
import { genreLabels } from "../../data/genres";
import UpcomingBadge from "./UpcomingBadge";
import { useUserData } from "../../context/UserDataContext";

// `action` choisit ce que fait le bouton sur l'affiche : aimer, ou retirer des
// films vus depuis l'onglet correspondant de Ma liste.
function PosterCard({ movie, action = "like" }) {
    const { isLiked, isWatched, toggleLike, toggleWatched } = useUserData();
    const onWatched = action === "watched";

    const active = onWatched ? isWatched(movie.id) : isLiked(movie.id);
    const Icon = onWatched ? Eye : Heart;
    const toggle = () => (onWatched ? toggleWatched(movie.id) : toggleLike(movie.id));
    const label = onWatched
        ? active
            ? `Retirer ${movie.title} de mes films vus`
            : `Marquer ${movie.title} comme vu`
        : active
          ? `Retirer ${movie.title} de mes aimés`
          : `Aimer ${movie.title}`;

    const poster = posterUrl(movie.poster_path);
    const rating = formatRating(movie.rating);
    const upcoming = isUpcoming(movie.release_date);
    // Un film à venir n'a pas à être signalé « Aucune note » : c'est normal,
    // et le badge de sortie le dit déjà.
    const showsNoRating = !rating && !upcoming;

    return (
        <article>
            <div className="relative aspect-2/3 rounded-poster overflow-hidden bg-surface">
                <Link
                    to={`/film/${movie.id}`}
                    aria-label={`Voir la fiche de ${movie.title}`}
                    className="block w-full h-full"
                >
                    {poster ? (
                        <img
                            src={poster}
                            alt=""
                            className="w-full h-full object-cover transition-transform duration-300 hover:scale-105"
                        />
                    ) : (
                        <div className="w-full h-full flex items-center justify-center text-edge">
                            <Film size={32} strokeWidth={1.5} aria-hidden="true" />
                        </div>
                    )}
                </Link>

                {upcoming && (
                    <UpcomingBadge
                        releaseDate={movie.release_date}
                        className="absolute top-2 left-2"
                    />
                )}

                <button
                    onClick={toggle}
                    aria-pressed={active}
                    aria-label={label}
                    className={`absolute top-2 right-2 w-11 h-11 rounded-pill flex items-center justify-center transition-colors ${
                        active ? "bg-gold text-on-gold" : "bg-base/70 text-ink hover:bg-base"
                    }`}
                >
                    <Icon
                        size={18}
                        strokeWidth={1.5}
                        fill={active && !onWatched ? "currentColor" : "none"}
                        aria-hidden="true"
                    />
                </button>
            </div>

            {/* font-sans : les h3 passent en Big Shoulders par défaut, trop lourd ici. */}
            <h3 className="font-sans font-semibold text-sm text-ink mt-3 leading-tight">
                {movie.title}
            </h3>
            <p className="text-xs text-muted mt-1 flex items-center gap-2">
                <span className="truncate">
                    {genreLabels(movie.genres?.slice(0, 2)).join(", ")}
                </span>
                {rating && (
                    <span className="ml-auto text-gold font-semibold shrink-0">{rating}</span>
                )}
                {showsNoRating && <span className="ml-auto shrink-0">Aucune note</span>}
            </p>
        </article>
    );
}

export default PosterCard;

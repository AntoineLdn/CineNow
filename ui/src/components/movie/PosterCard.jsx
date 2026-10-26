import { Link } from "react-router";
import { Heart, Film } from "lucide-react";
import { posterUrl } from "../../utils/poster";
import { formatRating } from "../../utils/format";
import { useUserData } from "../../context/UserDataContext";

function PosterCard({ movie }) {
    const { isLiked, toggleLike } = useUserData();
    const liked = isLiked(movie.id);
    const poster = posterUrl(movie.poster_path);

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

                <button
                    onClick={() => toggleLike(movie.id)}
                    aria-pressed={liked}
                    aria-label={
                        liked ? `Retirer ${movie.title} de mes aimés` : `Aimer ${movie.title}`
                    }
                    className={`absolute top-2 right-2 w-11 h-11 rounded-pill flex items-center justify-center transition-colors ${
                        liked ? "bg-gold text-on-gold" : "bg-base/70 text-ink hover:bg-base"
                    }`}
                >
                    <Heart
                        size={18}
                        strokeWidth={1.5}
                        fill={liked ? "currentColor" : "none"}
                        aria-hidden="true"
                    />
                </button>
            </div>

            {/* font-sans : les h3 passent en Big Shoulders par défaut, trop lourd ici. */}
            <h3 className="font-sans font-semibold text-sm text-ink mt-3 leading-tight">
                {movie.title}
            </h3>
            <p className="text-xs text-muted mt-1 flex items-center gap-2">
                <span className="truncate">{movie.genres?.slice(0, 2).join(", ")}</span>
                <span className="ml-auto text-gold font-semibold shrink-0">
                    {formatRating(movie.rating)}
                </span>
            </p>
        </article>
    );
}

export default PosterCard;

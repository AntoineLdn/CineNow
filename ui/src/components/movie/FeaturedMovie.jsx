import { useEffect, useState } from "react";
import { Link } from "react-router";
import { Heart, Eye, Star, Film } from "lucide-react";
import { posterUrl } from "../../utils/poster";
import { formatRating, formatDuration, isUpcoming } from "../../utils/format";
import { useUserData } from "../../context/UserDataContext";
import { getMovieDetails } from "../../api/client";
import ToggleButton from "../ui/ToggleButton";

function FeaturedMovie({ movie }) {
    const { isLiked, isWatched, toggleLike, toggleWatched } = useUserData();
    const [details, setDetails] = useState(null);

    // La durée et le synopsis ne transitent pas par MQTT : il faut les demander.
    useEffect(() => {
        let cancelled = false;
        setDetails(null);
        getMovieDetails(movie.id)
            .then((data) => {
                if (!cancelled) setDetails(data);
            })
            .catch(() => {
                // Sans détails, la fiche reste lisible : on masque durée et synopsis.
            });
        return () => {
            cancelled = true;
        };
    }, [movie.id]);

    const poster = posterUrl(movie.poster_path);
    const duration = formatDuration(details?.duration);
    const rating = formatRating(movie.rating);
    const showsNoRating = !rating && !isUpcoming(movie.release_date);

    return (
        <article className="bg-surface rounded-surface p-6 flex flex-col md:flex-row gap-7">
            <div className="w-full md:w-60 shrink-0 aspect-2/3 rounded-poster overflow-hidden bg-raised">
                {poster ? (
                    <img src={poster} alt="" className="w-full h-full object-cover" />
                ) : (
                    <div className="w-full h-full flex items-center justify-center text-edge">
                        <Film size={40} strokeWidth={1.5} aria-hidden="true" />
                    </div>
                )}
            </div>

            <div className="flex-1 flex flex-col">
                <h3 className="font-display text-5xl text-ink leading-none">{movie.title}</h3>

                <p className="flex flex-wrap items-center gap-x-4 gap-y-1 text-sm text-muted mt-3">
                    {movie.genres?.length > 0 && <span>{movie.genres.join(", ")}</span>}
                    {duration && <span>{duration}</span>}
                    {rating && (
                        <span className="flex items-center gap-1.5 text-gold font-semibold">
                            <Star
                                size={14}
                                strokeWidth={1.5}
                                fill="currentColor"
                                aria-hidden="true"
                            />
                            {rating}
                        </span>
                    )}
                    {showsNoRating && <span>Aucune note</span>}
                </p>

                {details?.overview && (
                    <p className="text-prose leading-relaxed mt-5">{details.overview}</p>
                )}

                <div className="flex flex-wrap gap-3 mt-auto pt-7">
                    <ToggleButton
                        active={isLiked(movie.id)}
                        Icon={Heart}
                        fillWhenActive
                        activeLabel="Dans mes aimés"
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
                    <Link
                        to={`/film/${movie.id}`}
                        className="h-pill px-6 rounded-pill bg-gold text-on-gold font-semibold flex items-center transition-opacity hover:opacity-90"
                    >
                        Voir la fiche
                    </Link>
                </div>
            </div>
        </article>
    );
}

export default FeaturedMovie;

import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faStar, faFire, faHeart, faEye } from "@fortawesome/free-solid-svg-icons";
import {
  faHeart as faHeartOutline,
  faEye as faEyeOutline,
} from "@fortawesome/free-regular-svg-icons";
import { posterUrl } from "../../utils/poster";
import { useUserData } from "../../context/UserDataContext";

// Les actions restent discrètes : visibles au survol, mais toujours affichées
// quand elles sont actives, sinon on ne verrait pas ce qu'on a déjà marqué.
function actionClass(active) {
  return `w-8 h-8 flex items-center justify-center rounded-lg bg-white/90 backdrop-blur-sm shadow-sm transition-all hover:bg-white hover:scale-110 ${
      active
          ? "text-orange-500"
          : "text-stone-400 opacity-0 group-hover:opacity-100 focus-visible:opacity-100"
  }`;
}

function MovieCard({ movie, onClick }) {
  const poster = posterUrl(movie.poster_path);
  const { isLiked, isWatched, toggleLike, toggleWatched } = useUserData();

  const liked = isLiked(movie.id);
  const watched = isWatched(movie.id);

  return (
      <div
          onClick={() => onClick?.(movie)}
          className="group flex flex-col bg-white rounded-2xl overflow-hidden shadow-sm hover:shadow-2xl transition-all duration-300 hover:-translate-y-2 cursor-pointer border border-stone-100"
      >
        <div className="relative aspect-2/3 overflow-hidden bg-stone-200">
          {poster ? (
              <img src={poster} alt={movie.title} className="w-full h-full object-cover transition-transform duration-500 group-hover:scale-110" />
          ) : (
              <div className="w-full h-full flex items-center justify-center text-stone-400">
                <FontAwesomeIcon icon={faFire} className="text-4xl" />
              </div>
          )}
          <div className="absolute inset-0 bg-linear-to-t from-black/80 via-black/20 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300" />

          {/* Badge genre */}
          {movie.genres?.length > 0 && (
              <div className="absolute top-3 left-3 px-2.5 py-1 bg-white/90 backdrop-blur-sm rounded-lg text-xs font-bold text-stone-800 shadow-sm capitalize">
                {movie.genres[0]}
              </div>
          )}

          {/* Badge année */}
          {movie.release_date && (
              <div className="absolute top-3 right-3 px-2.5 py-1 bg-white/90 backdrop-blur-sm rounded-lg text-xs font-bold text-stone-800 shadow-sm">
                {movie.release_date.slice(0, 4)}
              </div>
          )}

          {/* Actions : un clic ici ne doit pas ouvrir la fiche du film */}
          <div
              className="absolute bottom-3 right-3 flex gap-2"
              onClick={(e) => e.stopPropagation()}
          >
            <button
                onClick={() => toggleLike(movie.id)}
                className={actionClass(liked)}
                aria-pressed={liked}
                title={liked ? "Retirer des favoris" : "J'aime"}
            >
              <FontAwesomeIcon icon={liked ? faHeart : faHeartOutline} className="text-sm" />
            </button>
            <button
                onClick={() => toggleWatched(movie.id)}
                className={actionClass(watched)}
                aria-pressed={watched}
                title={watched ? "Marquer comme non vu" : "Déjà vu"}
            >
              <FontAwesomeIcon icon={watched ? faEye : faEyeOutline} className="text-sm" />
            </button>
          </div>
        </div>

        <div className="p-4 flex flex-col flex-1 justify-between">
          <h3 className="font-bold text-stone-800 leading-tight mb-1 line-clamp-2 group-hover:text-orange-600 transition-colors">
            {movie.title}
          </h3>
          <div className="flex items-center gap-1.5 mt-3">
            <FontAwesomeIcon icon={faStar} className="text-yellow-400 text-sm" />
            <span className="font-semibold text-stone-700 text-sm">{movie.rating.toFixed(1)}</span>
          </div>
        </div>
      </div>
  );
}

export default MovieCard;
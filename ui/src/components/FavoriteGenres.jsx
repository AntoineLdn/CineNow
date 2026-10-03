import { useUserData } from "../context/UserDataContext";

// Liste figée plutôt que saisie libre : les identifiants doivent correspondre
// aux genres normalisés publiés par le movie-service (voir GENRES dans
// services/movie-service/src/main.rs), sinon un favori ne matcherait aucun film.
const GENRES = [
  { id: "action", label: "Action" },
  { id: "animation", label: "Animation" },
  { id: "aventure", label: "Aventure" },
  { id: "comedie", label: "Comédie" },
  { id: "documentaire", label: "Documentaire" },
  { id: "drame", label: "Drame" },
  { id: "familial", label: "Familial" },
  { id: "fantastique", label: "Fantastique" },
  { id: "guerre", label: "Guerre" },
  { id: "histoire", label: "Histoire" },
  { id: "horreur", label: "Horreur" },
  { id: "romance", label: "Romance" },
  { id: "science-fiction", label: "Science-fiction" },
  { id: "thriller", label: "Thriller" },
];

function FavoriteGenres() {
  const { isFavoriteGenre, toggleFavoriteGenre } = useUserData();

  return (
      <div>
        <h3 className="text-xs font-bold text-stone-400 uppercase tracking-wider mb-3">
          Genres favoris
        </h3>
        <div className="flex flex-wrap gap-1.5">
          {GENRES.map((genre) => {
            const isFavorite = isFavoriteGenre(genre.id);
            return (
                <button
                    key={genre.id}
                    onClick={() => toggleFavoriteGenre(genre.id)}
                    aria-pressed={isFavorite}
                    className={`px-2.5 py-1 rounded-full text-xs font-semibold border transition-all
                  ${isFavorite
                        ? "bg-orange-50 border-orange-400 text-amber-700"
                        : "bg-stone-50 border-stone-200 text-stone-500 hover:border-orange-300 hover:bg-white"
                    }`}
                >
                  {genre.label}
                </button>
            );
          })}
        </div>
      </div>
  );
}

export default FavoriteGenres;
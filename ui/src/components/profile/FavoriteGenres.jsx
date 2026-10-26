import { useUserData } from "../../context/UserDataContext";

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

function countLine(count) {
    if (count === 0) return "Aucun genre ne pèse encore dans tes recommandations.";
    return count === 1
        ? "1 genre compte dans tes recommandations."
        : `${count} genres comptent dans tes recommandations.`;
}

function FavoriteGenres() {
    const { favoriteGenres, isFavoriteGenre, toggleFavoriteGenre } = useUserData();

    return (
        <section className="bg-surface rounded-surface p-8">
            <h2 className="text-section">Genres préférés</h2>
            <p className="text-muted mt-2">{countLine(favoriteGenres.length)}</p>

            <div className="flex flex-wrap gap-3 mt-6">
                {GENRES.map(({ id, label }) => {
                    const isFavorite = isFavoriteGenre(id);
                    return (
                        <button
                            key={id}
                            onClick={() => toggleFavoriteGenre(id)}
                            aria-pressed={isFavorite}
                            className={`h-pill px-5 rounded-pill border font-semibold transition-colors ${
                                isFavorite
                                    ? "bg-gold border-gold text-on-gold"
                                    : "border-edge text-ink hover:bg-raised"
                            }`}
                        >
                            {label}
                        </button>
                    );
                })}
            </div>
        </section>
    );
}

export default FavoriteGenres;

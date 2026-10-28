import { useUserData } from "../../context/UserDataContext";
import { SELECTABLE_GENRES, genreLabel } from "../../data/genres";

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
                {SELECTABLE_GENRES.map((genre) => {
                    const isFavorite = isFavoriteGenre(genre);
                    return (
                        <button
                            key={genre}
                            onClick={() => toggleFavoriteGenre(genre)}
                            aria-pressed={isFavorite}
                            className={`h-pill px-5 rounded-pill border font-semibold transition-colors ${
                                isFavorite
                                    ? "bg-gold border-gold text-on-gold"
                                    : "border-edge text-ink hover:bg-raised"
                            }`}
                        >
                            {genreLabel(genre)}
                        </button>
                    );
                })}
            </div>
        </section>
    );
}

export default FavoriteGenres;

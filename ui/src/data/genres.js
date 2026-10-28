// Libellés d'affichage des genres, indexés par le slug normalisé que renvoie le
// backend. Couvre les 19 genres de shared::TMDB_GENRE_MAP, pas seulement ceux
// que le profil propose : un film peut porter « crime » ou « western » sans
// qu'on puisse les mettre en favori.
const GENRE_LABELS = new Map([
    ["action", "Action"],
    ["animation", "Animation"],
    ["aventure", "Aventure"],
    ["comedie", "Comédie"],
    ["crime", "Crime"],
    ["documentaire", "Documentaire"],
    ["drame", "Drame"],
    ["familial", "Familial"],
    ["fantastique", "Fantastique"],
    ["guerre", "Guerre"],
    ["histoire", "Histoire"],
    ["horreur", "Horreur"],
    ["musique", "Musique"],
    ["mystere", "Mystère"],
    ["romance", "Romance"],
    ["science-fiction", "Science-fiction"],
    ["tele-film", "Téléfilm"],
    ["thriller", "Thriller"],
    ["western", "Western"],
]);

// Genres que le profil propose en favori : ceux que movie-service publie
// réellement (voir GENRES dans services/movie-service/src/main.rs). Mettre
// « western » en favori ne correspondrait à aucun film.
export const SELECTABLE_GENRES = [
    "action",
    "animation",
    "aventure",
    "comedie",
    "documentaire",
    "drame",
    "familial",
    "fantastique",
    "guerre",
    "histoire",
    "horreur",
    "romance",
    "science-fiction",
    "thriller",
];

// Un slug inconnu est capitalisé plutôt qu'affiché brut : si TMDb ajoute un
// genre, l'interface reste lisible en attendant la mise à jour de la table.
export function genreLabel(slug) {
    if (!slug) return "";
    return GENRE_LABELS.get(slug) ?? slug.charAt(0).toUpperCase() + slug.slice(1);
}

export function genreLabels(slugs) {
    return (slugs ?? []).map(genreLabel);
}

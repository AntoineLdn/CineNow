// Reconstruit l'URL d'affiche TMDb à partir du suffixe transmis par le backend.
// Le backend n'envoie que le suffixe (ex. "/abc123.jpg") pour alléger les
// messages MQTT ; le préfixe est constant et ajouté ici.
const TMDB_IMAGE_BASE = "https://image.tmdb.org/t/p/w500";

export function posterUrl(poster) {
    if (!poster) return null;
    // Tolère un ancien format déjà complet (URL absolue).
    if (poster.startsWith("http")) return poster;
    return `${TMDB_IMAGE_BASE}${poster}`;
}

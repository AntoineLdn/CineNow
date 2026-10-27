// Note sur 10, virgule décimale française : 7.8 -> « 7,8 ».
// Renvoie null quand aucun vote n'étaye la note : l'appelant affiche alors
// « Aucune note », plutôt qu'un « 0,0 » trompeur.
export function formatRating(rating) {
    if (rating === null || rating === undefined) return null;
    return rating.toFixed(1).replace(".", ",");
}

// Film pas encore sorti. Exige une date : sans elle on ne peut rien affirmer,
// et surtout rien afficher (le badge annonce « Sortie le … »).
//
// Volontairement différent de la règle d'exclusion côté serveur, qui écarte de
// la recommandation un film dont la date manque : là-bas on refuse de
// recommander sans preuve, ici on refuse d'annoncer sans date.
export function isUpcoming(releaseDate) {
    if (!releaseDate) return false;
    return releaseDate > new Date().toISOString().slice(0, 10);
}

// « Greta Lee » -> « GL », pour les pastilles de la distribution.
export function initials(name) {
    return name
        .split(/\s+/)
        .filter(Boolean)
        .slice(0, 2)
        .map((part) => part.charAt(0).toUpperCase())
        .join("");
}

// Durée en minutes -> « 1 h 46 ». Renvoie null si le backend ne la connaît pas.
export function formatDuration(minutes) {
    if (!minutes) return null;
    const hours = Math.floor(minutes / 60);
    const rest = minutes % 60;
    return hours ? `${hours} h ${String(rest).padStart(2, "0")}` : `${rest} min`;
}

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
// « 2026-11-12 » -> « 12 nov. ». L'année n'apparaît que si elle diffère de
// l'année en cours, sinon elle alourdit sans rien lever.
export function formatReleaseDate(releaseDate) {
    if (!releaseDate) return null;

    // Sans l'heure, la chaîne est lue comme minuit UTC et peut reculer d'un jour
    // selon le fuseau.
    const date = new Date(`${releaseDate}T00:00:00`);
    if (Number.isNaN(date.getTime())) return null;

    const options = { day: "numeric", month: "short" };
    if (date.getFullYear() !== new Date().getFullYear()) options.year = "numeric";

    return date.toLocaleDateString("fr-FR", options);
}

export function isUpcoming(releaseDate) {
    if (!releaseDate) return false;

    // Une date illisible ne doit pas compter comme « à venir » : le badge ne
    // saurait pas l'afficher, et on masquerait aussi « Aucune note ».
    const release = new Date(`${releaseDate}T00:00:00`);
    if (Number.isNaN(release.getTime())) return false;

    const today = new Date();
    today.setHours(0, 0, 0, 0);
    return release > today;
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

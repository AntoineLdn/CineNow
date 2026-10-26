// Note sur 10, virgule décimale française : 7.8 -> « 7,8 ».
export function formatRating(rating) {
    return rating.toFixed(1).replace(".", ",");
}

// Durée en minutes -> « 1 h 46 ». Renvoie null si le backend ne la connaît pas.
export function formatDuration(minutes) {
    if (!minutes) return null;
    const hours = Math.floor(minutes / 60);
    const rest = minutes % 60;
    return hours ? `${hours} h ${String(rest).padStart(2, "0")}` : `${rest} min`;
}

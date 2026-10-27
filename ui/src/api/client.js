// Point d'entrée unique vers le backend : aucune URL en dur ailleurs dans l'app.

const API_BASE = (import.meta.env.VITE_API_URL ?? "http://localhost:3001").replace(
    /\/$/,
    ""
);

// Porte le code HTTP, pour que l'appelant puisse réagir à un statut précis
// (c'est ce qui permet à auth.js de traiter le 401 à part).
export class ApiError extends Error {
    constructor(status, message) {
        super(message);
        this.name = "ApiError";
        this.status = status;
    }
}

async function parseError(res) {
    const text = await res.text();
    return text || `Erreur ${res.status}`;
}

export async function apiFetch(path, options = {}) {
    const res = await fetch(`${API_BASE}${path}`, options);
    if (!res.ok) throw new ApiError(res.status, await parseError(res));
    return res;
}

export async function apiJson(path, options) {
    const res = await apiFetch(path, options);
    return res.json();
}

export function postJson(path, body) {
    return apiFetch(path, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
    });
}

// Formes renvoyées par le backend, déclarées ici parce que c'est le seul endroit
// qui les lit. Les composants les récupèrent par inférence.

/**
 * @typedef {object} Weather
 * @property {number} temp
 * @property {string} condition
 * @property {string} period Matin, Après-midi, Soir ou Nuit
 * @property {string} city
 */

/**
 * @typedef {object} Movie
 * @property {number} id
 * @property {string} title
 * @property {string[]} genres
 * @property {number} rating
 * @property {string | null} poster_path
 * @property {string} overview vide quand le film vient du flux MQTT
 * @property {string | null} release_date
 */

/**
 * @typedef {Movie & {
 *   duration: number | null,
 *   director: string | null,
 *   actors: string[]
 * }} MovieDetails
 */

// ------------------------------------------------------------------- Météo

/** @returns {Promise<Weather>} */
export function getWeather() {
    return apiJson("/weather");
}

export function sendLocation(lat, lon) {
    return postJson("/location", { lat, lon });
}

// ------------------------------------------------------------------- Films

/** @returns {Promise<Record<string, Movie[]>>} films groupés par genre */
export function getMovies() {
    return apiJson("/movies");
}

/** @returns {Promise<MovieDetails>} */
export function getMovieDetails(movieId) {
    return apiJson(`/movie/${movieId}`);
}

// La route refuse plus de 100 ids par appel : on découpe, sinon une liste plus
// longue partirait en 400.
const MAX_IDS_PER_REQUEST = 100;

/**
 * Résout une liste d'identifiants en films. Les ids introuvables sont
 * simplement absents du résultat.
 *
 * @param {number[]} ids
 * @returns {Promise<Movie[]>}
 */
export async function getMoviesByIds(ids) {
    const chunks = [];
    for (let start = 0; start < ids.length; start += MAX_IDS_PER_REQUEST) {
        chunks.push(ids.slice(start, start + MAX_IDS_PER_REQUEST));
    }

    const pages = await Promise.all(
        chunks.map((chunk) => apiJson(`/movies/by-ids?ids=${chunk.join(",")}`))
    );

    return pages.flat();
}

// La recommandation vit dans auth.js : la route est authentifiée, donc elle
// passe par fetchAuth comme les autres.

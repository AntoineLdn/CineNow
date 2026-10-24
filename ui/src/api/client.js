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

// ------------------------------------------------------------------- Météo

export function getWeather() {
    return apiJson("/weather");
}

export function sendLocation(lat, lon) {
    return postJson("/location", { lat, lon });
}

// ------------------------------------------------------------------- Films

export function getMovies() {
    return apiJson("/movies");
}

export function getMovieDetails(movieId) {
    return apiJson(`/movie/${movieId}`);
}

// --------------------------------------------------------- Recommandations

// Le serveur ne répond pas la recommandation : il la publie sur MQTT, et le
// résultat n'apparaît sur /recommendations qu'une fois le calcul fait. D'où
// cette attente, seul endroit où elle vit — quand le backend renverra le
// résultat directement, cette fonction sera la seule à changer.
const RECOMMENDATION_DELAY_MS = 1000;

export async function getRecommendations(moods) {
    await postJson("/mood", { moods });
    await new Promise((resolve) => setTimeout(resolve, RECOMMENDATION_DELAY_MS));
    return apiJson("/recommendations");
}

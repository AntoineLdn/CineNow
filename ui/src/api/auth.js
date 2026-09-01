// Client d'authentification et d'accès à l'espace utilisateur.
//
// Le token JWT est conservé en mémoire (variable de module) et non dans
// localStorage : dans une app Tauri desktop, la mémoire du process suffit pour
// la session courante, et cela évite l'exposition XSS du token. Pour persister
// la session entre deux lancements, on branchera plus tard le store sécurisé
// de Tauri (plugin-store / keyring) plutôt que localStorage.

const API_BASE = "http://localhost:3001";

let authToken = null;

export function getToken() {
    return authToken;
}

export function isLoggedIn() {
    return authToken !== null;
}

export function logout() {
    authToken = null;
}

// En-têtes avec le token pour les routes protégées.
function authHeaders() {
    return authToken ? { Authorization: `Bearer ${authToken}` } : {};
}

async function parseError(res) {
    const text = await res.text();
    return text || `Erreur ${res.status}`;
}

export async function register(username, password) {
    const res = await fetch(`${API_BASE}/auth/register`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ username, password }),
    });
    if (!res.ok) throw new Error(await parseError(res));
    const data = await res.json();
    authToken = data.token;
    return data; // { token, username }
}

export async function login(username, password) {
    const res = await fetch(`${API_BASE}/auth/login`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ username, password }),
    });
    if (!res.ok) throw new Error(await parseError(res));
    const data = await res.json();
    authToken = data.token;
    return data;
}

// ---------------------------------------------------------------- Espace user

export async function getLikes() {
    const res = await fetch(`${API_BASE}/me/likes`, { headers: authHeaders() });
    if (!res.ok) throw new Error(await parseError(res));
    return res.json(); // number[]
}

export async function addLike(movieId) {
    const res = await fetch(`${API_BASE}/me/likes/${movieId}`, {
        method: "POST",
        headers: authHeaders(),
    });
    if (!res.ok) throw new Error(await parseError(res));
}

export async function removeLike(movieId) {
    const res = await fetch(`${API_BASE}/me/likes/${movieId}`, {
        method: "DELETE",
        headers: authHeaders(),
    });
    if (!res.ok) throw new Error(await parseError(res));
}

export async function getWatched() {
    const res = await fetch(`${API_BASE}/me/watched`, { headers: authHeaders() });
    if (!res.ok) throw new Error(await parseError(res));
    return res.json();
}

export async function addWatched(movieId) {
    const res = await fetch(`${API_BASE}/me/watched/${movieId}`, {
        method: "POST",
        headers: authHeaders(),
    });
    if (!res.ok) throw new Error(await parseError(res));
}

export async function removeWatched(movieId) {
    const res = await fetch(`${API_BASE}/me/watched/${movieId}`, {
        method: "DELETE",
        headers: authHeaders(),
    });
    if (!res.ok) throw new Error(await parseError(res));
}

export async function getFavoriteGenres() {
    const res = await fetch(`${API_BASE}/me/favorite-genres`, {
        headers: authHeaders(),
    });
    if (!res.ok) throw new Error(await parseError(res));
    return res.json(); // string[]
}

export async function addFavoriteGenre(genre) {
    const res = await fetch(`${API_BASE}/me/favorite-genres`, {
        method: "POST",
        headers: { "Content-Type": "application/json", ...authHeaders() },
        body: JSON.stringify({ genre }),
    });
    if (!res.ok) throw new Error(await parseError(res));
}

export async function removeFavoriteGenre(genre) {
    const res = await fetch(
        `${API_BASE}/me/favorite-genres/${encodeURIComponent(genre)}`,
        { method: "DELETE", headers: authHeaders() }
    );
    if (!res.ok) throw new Error(await parseError(res));
}

// Client d'authentification et d'accès à l'espace utilisateur.
//
// Le token JWT est confié au gestionnaire de secrets natif de l'OS via les
// commandes Tauri save_token / get_token / delete_token (voir
// src-tauri/src/lib.rs) : il n'est donc jamais écrit en clair sur le disque, ni
// exposé au XSS comme le serait localStorage. La session survit ainsi au
// redémarrage de l'app.
//
// Un cache mémoire double le keychain : on ne le consulte qu'une fois par
// lancement, plutôt qu'à chaque appel /me/*.

import { invoke } from "@tauri-apps/api/core";

const API_BASE = "http://localhost:3001";

let authToken = null;
let tokenLoaded = false; // le keychain a-t-il déjà été consulté ?

// Un keychain en erreur — ou l'absence d'IPC Tauri, quand le front est ouvert
// dans un navigateur nu via `npm run dev` — ne doit pas planter l'app : on
// retombe sur une session en mémoire seule, perdue à la fermeture.
async function safeInvoke(cmd, args) {
    try {
        return await invoke(cmd, args);
    } catch (e) {
        console.error(`Keychain indisponible (${cmd}) :`, e);
        return null;
    }
}

export async function getToken() {
    if (!tokenLoaded) {
        authToken = await safeInvoke("get_token");
        tokenLoaded = true;
    }
    return authToken;
}

export async function isLoggedIn() {
    return (await getToken()) !== null;
}

async function setToken(token) {
    authToken = token;
    tokenLoaded = true;
    await safeInvoke("save_token", { token });
}

export async function logout() {
    authToken = null;
    tokenLoaded = true; // inutile de relire le keychain : on vient de le vider
    await safeInvoke("delete_token");
}

// En-têtes avec le token pour les routes protégées.
async function authHeaders() {
    const token = await getToken();
    return token ? { Authorization: `Bearer ${token}` } : {};
}

async function parseError(res) {
    const text = await res.text();
    return text || `Erreur ${res.status}`;
}

// Prévenu quand le serveur rejette notre token : l'app remet l'écran de
// connexion. Passe par un callback plutôt qu'un import d'AuthContext, pour ne
// pas créer de dépendance circulaire entre le client et le contexte.
let unauthorizedHandler = null;

export function setUnauthorizedHandler(fn) {
    unauthorizedHandler = fn;
}

// Appel d'une route protégée : le token est attaché automatiquement, et un 401
// (secret JWT changé côté serveur, base réinitialisée, token révoqué) déclenche
// la déconnexion au lieu de laisser l'UI dans un état incohérent.
async function fetchAuth(path, options = {}) {
    const res = await fetch(`${API_BASE}${path}`, {
        ...options,
        headers: { ...(await authHeaders()), ...options.headers },
    });

    if (res.status === 401) {
        await logout();
        unauthorizedHandler?.();
        throw new Error("Session expirée, reconnecte-toi.");
    }

    if (!res.ok) throw new Error(await parseError(res));
    return res;
}

export async function register(username, password) {
    const res = await fetch(`${API_BASE}/auth/register`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ username, password }),
    });
    if (!res.ok) throw new Error(await parseError(res));
    const data = await res.json();
    await setToken(data.token);
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
    await setToken(data.token);
    return data;
}

// ---------------------------------------------------------------- Espace user

export async function getLikes() {
    const res = await fetchAuth("/me/likes");
    return res.json(); // number[]
}

export async function addLike(movieId) {
    await fetchAuth(`/me/likes/${movieId}`, { method: "POST" });
}

export async function removeLike(movieId) {
    await fetchAuth(`/me/likes/${movieId}`, { method: "DELETE" });
}

export async function getWatched() {
    const res = await fetchAuth("/me/watched");
    return res.json();
}

export async function addWatched(movieId) {
    await fetchAuth(`/me/watched/${movieId}`, { method: "POST" });
}

export async function removeWatched(movieId) {
    await fetchAuth(`/me/watched/${movieId}`, { method: "DELETE" });
}

export async function getFavoriteGenres() {
    const res = await fetchAuth("/me/favorite-genres");
    return res.json(); // string[]
}

export async function addFavoriteGenre(genre) {
    await fetchAuth("/me/favorite-genres", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ genre }),
    });
}

export async function removeFavoriteGenre(genre) {
    await fetchAuth(`/me/favorite-genres/${encodeURIComponent(genre)}`, {
        method: "DELETE",
    });
}
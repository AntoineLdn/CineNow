// Décodage du payload d'un JWT, SANS vérification de signature : c'est le
// serveur qui valide le token à chaque appel protégé. Ici on ne lit que
// `username` (pour l'affichage) et `exp` (pour ne pas restaurer une session
// déjà périmée).

function base64UrlDecode(segment) {
    const base64 = segment.replace(/-/g, "+").replace(/_/g, "/");
    const padded = base64.padEnd(Math.ceil(base64.length / 4) * 4, "=");
    const binary = atob(padded);
    const bytes = Uint8Array.from(binary, (c) => c.charCodeAt(0));
    // TextDecoder et pas atob seul : sinon un username accentué est illisible.
    return new TextDecoder().decode(bytes);
}

// Renvoie le payload, ou null si le token est malformé.
export function decodeJwt(token) {
    try {
        const payload = token.split(".")[1];
        if (!payload) return null;
        return JSON.parse(base64UrlDecode(payload));
    } catch {
        return null;
    }
}

// Un payload sans `exp` lisible est considéré comme périmé (on préfère
// redemander une connexion plutôt que de faire confiance à un token douteux).
export function isExpired(payload) {
    if (!payload || typeof payload.exp !== "number") return true;
    return payload.exp * 1000 <= Date.now();
}

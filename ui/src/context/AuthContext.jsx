import { createContext, useContext, useState, useCallback, useEffect } from "react";
import {
    login as apiLogin,
    register as apiRegister,
    logout as apiLogout,
    getToken,
    setUnauthorizedHandler,
} from "../api/auth";
import { decodeJwt, isExpired } from "../utils/jwt";

const AuthContext = createContext(null);

// Fournit l'état d'authentification (utilisateur connecté ou non) à toute l'app.
export function AuthProvider({ children }) {
    const [username, setUsername] = useState(null);
    // Vrai le temps de consulter le keychain : sans ça, le mur d'authentification
    // clignoterait à chaque lancement avant la restauration de la session.
    const [restoring, setRestoring] = useState(true);

    const login = useCallback(async (u, p) => {
        const data = await apiLogin(u, p);
        setUsername(data.username);
        return data;
    }, []);

    const register = useCallback(async (u, p) => {
        const data = await apiRegister(u, p);
        setUsername(data.username);
        return data;
    }, []);

    const logout = useCallback(async () => {
        setUsername(null);
        await apiLogout();
    }, []);

    // Un 401 sur une route protégée signifie que le serveur ne reconnaît plus
    // notre token : le client l'a déjà purgé du keychain, il reste à repasser
    // l'UI sur l'écran de connexion.
    useEffect(() => {
        setUnauthorizedHandler(() => setUsername(null));
        return () => setUnauthorizedHandler(null);
    }, []);

    // Restauration de session : un token encore valide dans le keychain
    // reconnecte l'utilisateur sans ressaisie.
    useEffect(() => {
        let cancelled = false;

        (async () => {
            const token = await getToken();
            const payload = token ? decodeJwt(token) : null;
            // On exige un username lisible : sans lui, rien à afficher, donc
            // autant traiter le token comme inutilisable.
            const valid =
                payload && !isExpired(payload) && typeof payload.username === "string";

            // Token périmé ou illisible : on purge l'entrée du keychain pour ne
            // pas retenter à chaque lancement.
            if (token && !valid) await apiLogout();

            if (cancelled) return;
            if (valid) setUsername(payload.username);
            setRestoring(false);
        })();

        return () => {
            cancelled = true;
        };
    }, []);

    const value = {
        username,
        isLoggedIn: username !== null,
        restoring,
        login,
        register,
        logout,
    };

    return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
    const ctx = useContext(AuthContext);
    if (!ctx) throw new Error("useAuth doit être utilisé dans un AuthProvider");
    return ctx;
}

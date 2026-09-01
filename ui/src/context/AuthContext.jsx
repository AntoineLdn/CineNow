import { createContext, useContext, useState, useCallback } from "react";
import { login as apiLogin, register as apiRegister, logout as apiLogout } from "../api/auth";

const AuthContext = createContext(null);

// Fournit l'état d'authentification (utilisateur connecté ou non) à toute l'app.
export function AuthProvider({ children }) {
    const [username, setUsername] = useState(null);

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

    const logout = useCallback(() => {
        apiLogout();
        setUsername(null);
    }, []);

    const value = {
        username,
        isLoggedIn: username !== null,
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

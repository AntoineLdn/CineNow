import { useState } from "react";
import { login, register } from "../api/auth";

// Panneau de connexion / inscription minimal.
// onAuth(username) est appelé après une authentification réussie.
function AuthPanel({ onAuth }) {
    const [mode, setMode] = useState("login"); // "login" | "register"
    const [username, setUsername] = useState("");
    const [password, setPassword] = useState("");
    const [error, setError] = useState(null);
    const [loading, setLoading] = useState(false);

    const submit = async () => {
        setError(null);
        setLoading(true);
        try {
            const action = mode === "login" ? login : register;
            const data = await action(username, password);
            onAuth?.(data.username);
        } catch (e) {
            setError(e.message);
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="max-w-sm mx-auto p-6 bg-white rounded-2xl shadow-sm border border-stone-100">
            <h2 className="text-xl font-bold text-stone-800 mb-4">
                {mode === "login" ? "Connexion" : "Créer un compte"}
            </h2>

            <input
                type="text"
                placeholder="Nom d'utilisateur"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                className="w-full mb-3 px-4 py-2 rounded-lg border border-stone-200 focus:outline-none focus:ring-2 focus:ring-orange-400"
            />
            <input
                type="password"
                placeholder="Mot de passe"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && submit()}
                className="w-full mb-3 px-4 py-2 rounded-lg border border-stone-200 focus:outline-none focus:ring-2 focus:ring-orange-400"
            />

            {error && <p className="text-red-600 text-sm mb-3">{error}</p>}

            <button
                onClick={submit}
                disabled={loading}
                className="w-full py-2 rounded-lg bg-orange-500 text-white font-semibold hover:bg-orange-600 transition-colors disabled:opacity-50"
            >
                {loading ? "..." : mode === "login" ? "Se connecter" : "S'inscrire"}
            </button>

            <button
                onClick={() => {
                    setMode(mode === "login" ? "register" : "login");
                    setError(null);
                }}
                className="w-full mt-3 text-sm text-stone-500 hover:text-orange-600"
            >
                {mode === "login"
                    ? "Pas de compte ? Créer un compte"
                    : "Déjà un compte ? Se connecter"}
            </button>
        </div>
    );
}

export default AuthPanel;

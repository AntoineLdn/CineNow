import { useState } from "react";
import { useAuth } from "../context/AuthContext";

// Panneau de connexion / inscription affiché tant que l'utilisateur n'est pas connecté.
function AuthPanel() {
    const { login, register } = useAuth();
    const [mode, setMode] = useState("login"); // "login" | "register"
    const [username, setUsername] = useState("");
    const [password, setPassword] = useState("");
    const [error, setError] = useState(null);
    const [loading, setLoading] = useState(false);

    const submit = async () => {
        setError(null);
        setLoading(true);
        try {
            if (mode === "login") {
                await login(username, password);
            } else {
                await register(username, password);
            }
            // Pas besoin de rediriger : le contexte passe isLoggedIn à true,
            // et App affiche automatiquement l'application.
        } catch (e) {
            setError(e.message);
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="flex items-center justify-center h-screen bg-stone-50">
            <div className="max-w-sm w-full p-8 bg-white rounded-2xl shadow-sm border border-stone-100">
                <h2 className="text-2xl font-bold text-stone-800 mb-1">Ciné-Now</h2>
                <p className="text-stone-500 mb-6">
                    {mode === "login" ? "Connectez-vous pour continuer" : "Créez votre compte"}
                </p>

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
                    className="w-full py-2.5 rounded-lg bg-orange-500 text-white font-semibold hover:bg-orange-600 transition-colors disabled:opacity-50"
                >
                    {loading ? "..." : mode === "login" ? "Se connecter" : "S'inscrire"}
                </button>

                <button
                    onClick={() => {
                        setMode(mode === "login" ? "register" : "login");
                        setError(null);
                    }}
                    className="w-full mt-4 text-sm text-stone-500 hover:text-orange-600"
                >
                    {mode === "login"
                        ? "Pas de compte ? Créer un compte"
                        : "Déjà un compte ? Se connecter"}
                </button>
            </div>
        </div>
    );
}

export default AuthPanel;

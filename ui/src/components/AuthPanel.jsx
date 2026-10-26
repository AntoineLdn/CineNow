import { useState } from "react";
import { useAuth } from "../context/AuthContext";

const FIELD_CLASS =
    "w-full h-pill px-5 rounded-pill bg-raised border border-edge text-ink placeholder:text-muted transition-colors focus:border-gold";

// Panneau de connexion / inscription affiché tant que l'utilisateur n'est pas connecté.
function AuthPanel() {
    const { login, register } = useAuth();
    const [mode, setMode] = useState("login"); // "login" | "register"
    const [username, setUsername] = useState("");
    const [password, setPassword] = useState("");
    const [error, setError] = useState(null);
    const [loading, setLoading] = useState(false);

    const isRegister = mode === "register";

    const submit = async (event) => {
        event.preventDefault();
        setError(null);
        setLoading(true);
        try {
            // Pas besoin de rediriger : le contexte passe isLoggedIn à true,
            // et App affiche automatiquement l'application.
            await (isRegister ? register : login)(username, password);
        } catch (e) {
            setError(e.message);
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="min-h-screen bg-base flex flex-col items-center justify-center p-8">
            <h1 className="text-greeting leading-none text-center">CinéNow</h1>
            <p className="text-muted mt-4 text-center">
                {isRegister
                    ? "Crée ton compte pour garder tes films."
                    : "Connecte-toi pour retrouver ta sélection."}
            </p>

            <form
                onSubmit={submit}
                className="w-full max-w-sm bg-surface rounded-surface p-8 mt-10 flex flex-col gap-5"
            >
                <div>
                    <label htmlFor="username" className="block text-sm font-semibold text-ink mb-2">
                        Nom d'utilisateur
                    </label>
                    <input
                        id="username"
                        type="text"
                        value={username}
                        onChange={(e) => setUsername(e.target.value)}
                        autoComplete="username"
                        className={FIELD_CLASS}
                    />
                </div>

                <div>
                    <label htmlFor="password" className="block text-sm font-semibold text-ink mb-2">
                        Mot de passe
                    </label>
                    <input
                        id="password"
                        type="password"
                        value={password}
                        onChange={(e) => setPassword(e.target.value)}
                        autoComplete={isRegister ? "new-password" : "current-password"}
                        className={FIELD_CLASS}
                    />
                    {isRegister && <p className="text-xs text-muted mt-2">8 caractères minimum.</p>}
                </div>

                {error && (
                    <p role="alert" className="text-sm text-gold">
                        {error}
                    </p>
                )}

                <button
                    type="submit"
                    disabled={loading}
                    className="h-pill px-6 rounded-pill bg-gold text-on-gold font-semibold transition-opacity hover:opacity-90 disabled:opacity-40"
                >
                    {loading ? "..." : isRegister ? "Créer mon compte" : "Se connecter"}
                </button>

                <button
                    type="button"
                    onClick={() => {
                        setMode(isRegister ? "login" : "register");
                        setError(null);
                    }}
                    className="text-sm text-muted min-h-11 transition-colors hover:text-ink"
                >
                    {isRegister ? "Déjà un compte ? Se connecter" : "Pas de compte ? En créer un"}
                </button>
            </form>
        </div>
    );
}

export default AuthPanel;

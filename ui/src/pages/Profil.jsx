import { useAuth } from "../context/AuthContext";

// Les genres favoris et le récap de la liste arrivent à une étape ultérieure ;
// la déconnexion est ici dès maintenant pour qu'elle reste atteignable.
function Profil() {
    const { username, logout } = useAuth();

    return (
        <div className="p-12">
            <h1 className="text-section">Profil</h1>
            <p className="text-muted mt-4">
                Connecté en tant que <span className="text-ink font-semibold">{username}</span>
            </p>

            <button
                onClick={logout}
                className="mt-8 h-pill px-6 rounded-pill bg-gold text-on-gold font-semibold transition-opacity hover:opacity-90"
            >
                Se déconnecter
            </button>
        </div>
    );
}

export default Profil;

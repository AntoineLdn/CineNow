import { Heart, Eye } from "lucide-react";
import { useAuth } from "../context/AuthContext";
import { useUserData } from "../context/UserDataContext";
import { initials } from "../utils/format";
import FavoriteGenres from "../components/profile/FavoriteGenres";

function StatTile({ Icon, value, label }) {
    return (
        <div className="bg-raised rounded-poster p-5 flex-1">
            <Icon size={20} strokeWidth={1.5} className="text-gold" aria-hidden="true" />
            <p className="font-display text-5xl text-ink mt-3">{value}</p>
            <p className="text-sm text-muted mt-1">{label}</p>
        </div>
    );
}

function Profil() {
    const { username, logout } = useAuth();
    const { likedIds, watchedIds } = useUserData();

    return (
        <div className="p-12 max-w-7xl">
            <header className="flex items-center gap-6">
                <span
                    aria-hidden="true"
                    className="w-20 h-20 rounded-pill bg-gold text-on-gold font-display text-4xl flex items-center justify-center shrink-0"
                >
                    {username ? initials(username) : "?"}
                </span>
                <div className="min-w-0">
                    <h1 className="text-greeting leading-none truncate">{username}</h1>
                    <p className="text-muted mt-2">
                        Tes goûts affinent les films proposés dans Ce soir.
                    </p>
                </div>
            </header>

            <div className="flex flex-col lg:flex-row gap-6 mt-12 items-start">
                <div className="flex-1 w-full">
                    <FavoriteGenres />
                </div>

                <div className="w-full lg:w-80 shrink-0 flex flex-col gap-6">
                    <section className="bg-surface rounded-surface p-8">
                        <h2 className="font-sans text-lg font-bold text-ink">Ta liste</h2>
                        <div className="flex gap-4 mt-5">
                            <StatTile Icon={Heart} value={likedIds.length} label="films aimés" />
                            <StatTile Icon={Eye} value={watchedIds.length} label="films vus" />
                        </div>
                    </section>

                    <section className="bg-surface rounded-surface p-8">
                        <h2 className="font-sans text-lg font-bold text-ink">Compte</h2>
                        <div className="flex items-center justify-between gap-4 mt-5">
                            <p className="text-muted truncate">{username}</p>
                            <button
                                onClick={logout}
                                className="text-gold font-semibold shrink-0 min-h-11"
                            >
                                Se déconnecter
                            </button>
                        </div>
                    </section>
                </div>
            </div>
        </div>
    );
}

export default Profil;

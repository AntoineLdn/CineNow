import { createContext, useContext, useState, useEffect, useCallback, useRef } from "react";
import { useAuth } from "./AuthContext";
import {
    getLikes,
    addLike,
    removeLike,
    getWatched,
    addWatched,
    removeWatched,
    getFavoriteGenres,
    addFavoriteGenre,
    removeFavoriteGenre,
} from "../api/auth";

const UserDataContext = createContext(null);

// Renvoie une copie du Set avec `id` ajouté ou retiré.
function withId(set, id, present) {
    const next = new Set(set);
    if (present) next.add(id);
    else next.delete(id);
    return next;
}

// Bascule optimiste sur un Set d'ids : l'affichage change tout de suite, l'appel
// part ensuite, et on revient à l'état précédent si le serveur refuse — mais
// seulement si la session n'a pas changé entre-temps (`isCurrent`).
async function toggleId(id, wasActive, setState, apiCall, message, isCurrent) {
    setState((prev) => withId(prev, id, !wasActive));
    try {
        await apiCall(id);
    } catch (e) {
        console.error(message, e);
        if (isCurrent()) setState((prev) => withId(prev, id, wasActive));
    }
}

// Centralise likes / films vus / genres favoris : un seul chargement à la
// connexion, au lieu d'un appel par carte affichée.
export function UserDataProvider({ children }) {
    const { isLoggedIn } = useAuth();
    const [likes, setLikes] = useState(() => new Set());
    const [watched, setWatched] = useState(() => new Set());
    const [favoriteGenres, setFavoriteGenres] = useState([]);

    // Incrémenté à chaque remise à zéro. Un appel parti avant la déconnexion
    // peut échouer après : son rollback ne doit pas réécrire dans les données
    // de la session suivante.
    const session = useRef(0);

    useEffect(() => {
        // Déconnexion : on vide, pour ne pas montrer les données du compte
        // précédent au suivant.
        if (!isLoggedIn) {
            session.current += 1;
            setLikes(new Set());
            setWatched(new Set());
            setFavoriteGenres([]);
            return;
        }

        let cancelled = false;

        (async () => {
            try {
                const [likeIds, watchedIds, genres] = await Promise.all([
                    getLikes(),
                    getWatched(),
                    getFavoriteGenres(),
                ]);
                if (cancelled) return;
                setLikes(new Set(likeIds));
                setWatched(new Set(watchedIds));
                setFavoriteGenres(genres);
            } catch (e) {
                // L'app reste utilisable sans l'espace utilisateur : les boutons
                // apparaîtront simplement inactifs.
                console.error("Chargement de l'espace utilisateur impossible :", e);
            }
        })();

        return () => {
            cancelled = true;
        };
    }, [isLoggedIn]);

    const toggleLike = useCallback(
        (movieId) => {
            const startSession = session.current;
            const wasLiked = likes.has(movieId);
            return toggleId(
                movieId,
                wasLiked,
                setLikes,
                wasLiked ? removeLike : addLike,
                "Like non enregistré :",
                () => session.current === startSession
            );
        },
        [likes]
    );

    const toggleWatched = useCallback(
        (movieId) => {
            const startSession = session.current;
            const wasWatched = watched.has(movieId);
            return toggleId(
                movieId,
                wasWatched,
                setWatched,
                wasWatched ? removeWatched : addWatched,
                "Film vu non enregistré :",
                () => session.current === startSession
            );
        },
        [watched]
    );

    const toggleFavoriteGenre = useCallback(
        async (genre) => {
            const startSession = session.current;
            const previous = favoriteGenres;
            const wasFavorite = previous.includes(genre);
            setFavoriteGenres(
                wasFavorite ? previous.filter((g) => g !== genre) : [...previous, genre]
            );
            try {
                await (wasFavorite ? removeFavoriteGenre : addFavoriteGenre)(genre);
            } catch (e) {
                console.error("Genre favori non enregistré :", e);
                if (session.current === startSession) setFavoriteGenres(previous);
            }
        },
        [favoriteGenres]
    );

    const value = {
        isLiked: (movieId) => likes.has(movieId),
        isWatched: (movieId) => watched.has(movieId),
        toggleLike,
        toggleWatched,
        favoriteGenres,
        isFavoriteGenre: (genre) => favoriteGenres.includes(genre),
        toggleFavoriteGenre,
    };

    return (
        <UserDataContext.Provider value={value}>{children}</UserDataContext.Provider>
    );
}

export function useUserData() {
    const ctx = useContext(UserDataContext);
    if (!ctx) throw new Error("useUserData doit être utilisé dans un UserDataProvider");
    return ctx;
}

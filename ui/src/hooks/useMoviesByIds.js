import { useEffect, useState } from "react";
import { getMovieDetails } from "../api/client";

// Les routes /me/* ne renvoient que des identifiants : il faut demander chaque
// film séparément, /movie/:id étant la seule route qui résout un id. Le cache
// évite de tout recharger à chaque changement d'onglet.
const cache = new Map();

function load(movieId) {
    if (!cache.has(movieId)) {
        // Un échec ne doit pas rester en cache, sinon l'id est perdu pour la
        // durée de la session.
        cache.set(
            movieId,
            getMovieDetails(movieId).catch((e) => {
                cache.delete(movieId);
                throw e;
            })
        );
    }
    return cache.get(movieId);
}

export function useMoviesByIds(ids) {
    const [movies, setMovies] = useState([]);
    const [status, setStatus] = useState("loading"); // loading | done | error

    // Dépendance par contenu : la liste est recréée à chaque rendu du contexte.
    const key = ids.join(",");

    useEffect(() => {
        const wanted = key ? key.split(",").map(Number) : [];

        if (wanted.length === 0) {
            setMovies([]);
            setStatus("done");
            return;
        }

        let cancelled = false;

        // Retirer un film de la liste relance cet effet. Sans ce test, la grille
        // repasserait par « chargement » alors que tout est déjà en cache.
        if (!wanted.every((movieId) => cache.has(movieId))) setStatus("loading");

        Promise.allSettled(wanted.map(load)).then((settled) => {
            if (cancelled) return;
            // allSettled et pas all : un film introuvable ne doit pas vider
            // toute la liste.
            const loaded = settled
                .filter((result) => result.status === "fulfilled")
                .map((result) => result.value);

            setMovies(loaded);
            setStatus(loaded.length === 0 ? "error" : "done");
        });

        return () => {
            cancelled = true;
        };
    }, [key]);

    return { movies, status };
}

import { useEffect, useState } from "react";
import { getMoviesByIds } from "../api/client";

// Les routes /me/* ne renvoient que des identifiants : /movies/by-ids les résout
// en une requête. Le cache évite de tout redemander à chaque changement
// d'onglet.
const cache = new Map();

// Ids que le serveur ne résout pas : sans cette trace, ils seraient redemandés à
// chaque passage puisqu'ils n'entrent jamais dans le cache.
const unresolved = new Set();

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

        // Les films connus, dans l'ordre demandé. Les absents sautent, ce qui
        // couvre les ids introuvables.
        const collect = () => wanted.map((movieId) => cache.get(movieId)).filter(Boolean);

        const missing = wanted.filter(
            (movieId) => !cache.has(movieId) && !unresolved.has(movieId)
        );

        // Retirer un film de la liste relance cet effet. Sans ce test, la grille
        // repasserait par « chargement » alors que tout est déjà en cache.
        if (missing.length === 0) {
            setMovies(collect());
            setStatus("done");
            return;
        }

        let cancelled = false;
        setStatus("loading");

        getMoviesByIds(missing)
            .then((fetched) => {
                if (cancelled) return;

                for (const movie of fetched) {
                    cache.set(movie.id, movie);
                }
                for (const movieId of missing) {
                    if (!cache.has(movieId)) unresolved.add(movieId);
                }

                setMovies(collect());
                setStatus("done");
            })
            .catch((e) => {
                // Échec réseau ou HTTP : rien n'est mémorisé, un prochain passage
                // retentera.
                console.error("Chargement des films impossible :", e);
                if (!cancelled) setStatus("error");
            });

        return () => {
            cancelled = true;
        };
    }, [key]);

    return { movies, status };
}

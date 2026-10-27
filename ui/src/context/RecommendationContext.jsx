import { createContext, useCallback, useContext, useEffect, useState } from "react";
import { useAuth } from "./AuthContext";
import { getRecommendations } from "../api/auth";

const RecommendationContext = createContext(null);

const GENERIC_ERROR = "La recommandation n'a pas abouti.";

// Les deux cas d'attente côté serveur méritent un message qui dise quoi faire,
// plutôt que l'erreur générique.
function errorMessage(error) {
    switch (error?.status) {
        case 504:
            return "Le calcul a pris trop de temps, réessaie.";
        case 503:
            return "Les films sont encore en cours de chargement, réessaie dans un instant.";
        // Humeur vide ou inconnue : le serveur dit laquelle.
        case 400:
            return error.message || GENERIC_ERROR;
        default:
            return GENERIC_ERROR;
    }
}

// La sélection vit hors de la page : ouvrir une fiche démonte Ce soir, et on
// doit retrouver les films recommandés en revenant, pas un écran vierge.
export function RecommendationProvider({ children }) {
    const { isLoggedIn } = useAuth();
    const [selectedMoods, setSelectedMoods] = useState([]);
    const [results, setResults] = useState([]);
    const [status, setStatus] = useState("idle"); // idle | loading | done | error
    const [error, setError] = useState(null);

    useEffect(() => {
        if (!isLoggedIn) {
            setSelectedMoods([]);
            setResults([]);
            setStatus("idle");
            setError(null);
        }
    }, [isLoggedIn]);

    const toggleMood = useCallback((moodId) => {
        setSelectedMoods((prev) =>
            prev.includes(moodId) ? prev.filter((id) => id !== moodId) : [...prev, moodId]
        );
    }, []);

    const recommend = useCallback(async () => {
        if (selectedMoods.length === 0) return;
        setStatus("loading");
        setError(null);
        try {
            setResults(await getRecommendations(selectedMoods));
            setStatus("done");
        } catch (e) {
            setError(errorMessage(e));
            setStatus("error");
        }
    }, [selectedMoods]);

    const value = { selectedMoods, toggleMood, results, status, error, recommend };

    return (
        <RecommendationContext.Provider value={value}>{children}</RecommendationContext.Provider>
    );
}

export function useRecommendations() {
    const ctx = useContext(RecommendationContext);
    if (!ctx) {
        throw new Error("useRecommendations doit être utilisé dans un RecommendationProvider");
    }
    return ctx;
}

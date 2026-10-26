import { createContext, useCallback, useContext, useEffect, useState } from "react";
import { useAuth } from "./AuthContext";
import { getRecommendations } from "../api/client";

const RecommendationContext = createContext(null);

// La sélection vit hors de la page : ouvrir une fiche démonte Ce soir, et on
// doit retrouver les films recommandés en revenant, pas un écran vierge.
export function RecommendationProvider({ children }) {
    const { isLoggedIn } = useAuth();
    const [selectedMoods, setSelectedMoods] = useState([]);
    const [results, setResults] = useState([]);
    const [status, setStatus] = useState("idle"); // idle | loading | done | error

    useEffect(() => {
        if (!isLoggedIn) {
            setSelectedMoods([]);
            setResults([]);
            setStatus("idle");
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
        try {
            setResults(await getRecommendations(selectedMoods));
            setStatus("done");
        } catch {
            setStatus("error");
        }
    }, [selectedMoods]);

    const value = { selectedMoods, toggleMood, results, status, recommend };

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

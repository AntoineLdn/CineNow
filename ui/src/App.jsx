import { useState } from "react";
import Sidebar from "./components/Sidebar";
import Home from "./pages/Home";
import AuthPanel from "./components/AuthPanel";
import { useAuth } from "./context/AuthContext";

function App() {
  const { isLoggedIn, restoring } = useAuth();
  const [selectedMoods, setSelectedMoods] = useState([]);
  const [recommendations, setRecommendations] = useState(null); // null = pas encore demandé

  const toggleMood = (moodId) => {
    setSelectedMoods((prev) =>
        prev.includes(moodId) ? prev.filter((id) => id !== moodId) : [...prev, moodId]
    );
  };

  const handleRecommend = async () => {
    if (selectedMoods.length === 0) return;

    await fetch("http://localhost:3001/mood", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ moods: selectedMoods }),
    });

    // On attend que le recommendation-service ait eu le temps de calculer
    setTimeout(async () => {
      const res = await fetch("http://localhost:3001/recommendations");
      const data = await res.json();
      setRecommendations(data);
    }, 1000);
  };

  const handleReset = () => {
    setSelectedMoods([]);
    setRecommendations(null);
  };

  // Lecture du keychain en cours : on n'affiche ni l'app ni le mur d'auth, sinon
  // ce dernier apparaîtrait une fraction de seconde à chaque lancement.
  if (restoring) {
    return (
        <div className="flex items-center justify-center h-screen bg-stone-50 text-stone-400">
          Ouverture de la session...
        </div>
    );
  }

  // Mur d'authentification : tant que non connecté, on n'affiche que le panneau.
  if (!isLoggedIn) {
    return <AuthPanel />;
  }

  return (
      <div className="flex h-screen bg-stone-50 text-stone-800">
        <Sidebar
            selectedMoods={selectedMoods}
            onToggle={toggleMood}
            onRecommend={handleRecommend}
            onReset={handleReset}
        />
        <main className="flex-1 overflow-y-scroll">
          <Home recommendations={recommendations} />
        </main>
      </div>
  );
}

export default App;

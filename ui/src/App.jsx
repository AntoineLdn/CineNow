import { RouterProvider } from "react-router";
import AuthPanel from "./components/AuthPanel";
import { useAuth } from "./context/AuthContext";
import { router } from "./routes";

function App() {
  const { isLoggedIn, restoring } = useAuth();

  // Lecture du keychain en cours : on n'affiche ni l'app ni le mur d'auth, sinon
  // ce dernier apparaîtrait une fraction de seconde à chaque lancement.
  if (restoring) {
    return (
        <div className="flex items-center justify-center h-screen bg-base text-muted">
          Ouverture de la session...
        </div>
    );
  }

  // Mur d'authentification : tant que non connecté, on n'affiche que le panneau.
  if (!isLoggedIn) {
    return <AuthPanel />;
  }

  return <RouterProvider router={router} />;
}

export default App;

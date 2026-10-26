import { createHashRouter } from "react-router";
import AppLayout from "./components/layout/AppLayout";
import CeSoir from "./pages/CeSoir";
import Catalogue from "./pages/Catalogue";
import MaListe from "./pages/MaListe";
import Profil from "./pages/Profil";
import FilmPage from "./pages/FilmPage";

// Routage par hash : l'app est servie depuis un fichier local sous Tauri, où un
// historique par chemin ne retomberait pas sur index.html au rechargement.
export const router = createHashRouter([
    {
        element: <AppLayout />,
        children: [
            { index: true, element: <CeSoir /> },
            { path: "catalogue", element: <Catalogue /> },
            { path: "liste", element: <MaListe /> },
            { path: "profil", element: <Profil /> },
            { path: "film/:id", element: <FilmPage /> },
        ],
    },
]);

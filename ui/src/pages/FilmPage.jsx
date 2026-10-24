import { useParams } from "react-router";

// Remplace MovieModal à une étape ultérieure : affiche, distribution, actions.
function FilmPage() {
    const { id } = useParams();

    return (
        <div className="p-12">
            <h1 className="text-section">Fiche film</h1>
            <p className="text-muted mt-4">Détails du film {id} à venir.</p>
        </div>
    );
}

export default FilmPage;

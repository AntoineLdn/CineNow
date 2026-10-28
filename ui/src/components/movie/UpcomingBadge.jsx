import { formatReleaseDate } from "../../utils/format";

// Signale un film pas encore sorti. C'est aussi ce qui explique son absence de
// note : les deux vont ensemble, d'où la date plutôt qu'un simple « À venir ».
function UpcomingBadge({ releaseDate, className = "" }) {
    const date = formatReleaseDate(releaseDate);
    if (!date) return null;

    return (
        <span
            className={`px-3 py-1 rounded-pill bg-base/85 backdrop-blur-sm border border-edge text-xs text-ink whitespace-nowrap ${className}`}
        >
            Sortie le {date}
        </span>
    );
}

export default UpcomingBadge;

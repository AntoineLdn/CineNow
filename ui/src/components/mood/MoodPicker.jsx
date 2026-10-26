import { Sun, Droplet, Flame, Waves, Moon, Ghost, Compass, Lightbulb } from "lucide-react";

// Les identifiants doivent rester ceux attendus par la matrice d'affinité
// (sections de shared/src/data/affinities.toml).
const MOODS = [
    { id: "joy", label: "Joyeux", Icon: Sun },
    { id: "sad", label: "Triste", Icon: Droplet },
    { id: "angry", label: "Énervé", Icon: Flame },
    { id: "calm", label: "Calme", Icon: Waves },
    { id: "tired", label: "Fatigué", Icon: Moon },
    { id: "fear", label: "Peur", Icon: Ghost },
    { id: "adventure", label: "Aventure", Icon: Compass },
    { id: "reflection", label: "Réflexion", Icon: Lightbulb },
];

function MoodPicker({ selectedMoods, onToggle }) {
    return (
        <div>
            <h2 className="text-section">Comment tu te sens ?</h2>
            <p className="text-muted mt-2">Choisis une ou plusieurs humeurs.</p>

            <div className="flex flex-wrap gap-3 mt-6">
                {MOODS.map(({ id, label, Icon }) => {
                    const isSelected = selectedMoods.includes(id);
                    return (
                        <button
                            key={id}
                            onClick={() => onToggle(id)}
                            aria-pressed={isSelected}
                            className={`h-pill px-5 rounded-pill border font-semibold transition-colors ${
                                isSelected
                                    ? "bg-gold border-gold text-on-gold"
                                    : "border-edge text-ink hover:bg-surface"
                            }`}
                        >
                            <span className="flex items-center gap-2">
                                <Icon size={18} strokeWidth={1.5} aria-hidden="true" />
                                {label}
                            </span>
                        </button>
                    );
                })}
            </div>
        </div>
    );
}

export default MoodPicker;

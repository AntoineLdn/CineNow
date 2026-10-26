// Pastille d'action à deux états, partagée par la fiche film et le film mis en
// avant : dorée quand l'état est actif, contour sinon.
function ToggleButton({ active, Icon, activeLabel, idleLabel, onClick }) {
    return (
        <button
            onClick={onClick}
            aria-pressed={active}
            className={`h-pill px-5 rounded-pill border font-semibold transition-colors ${
                active ? "bg-gold border-gold text-on-gold" : "border-edge text-ink hover:bg-raised"
            }`}
        >
            <span className="flex items-center gap-2">
                <Icon
                    size={18}
                    strokeWidth={1.5}
                    fill={active ? "currentColor" : "none"}
                    aria-hidden="true"
                />
                {active ? activeLabel : idleLabel}
            </span>
        </button>
    );
}

export default ToggleButton;

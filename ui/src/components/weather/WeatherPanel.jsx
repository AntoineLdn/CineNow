import { MapPin, LocateFixed } from "lucide-react";
import { weatherLook } from "../../data/weather";

function WeatherPanel({ weather, onUseLocation, locating, locationError }) {
    const { Icon, tint } = weatherLook(weather.condition);

    return (
        <section
            aria-label="Météo actuelle"
            className={`relative overflow-hidden rounded-surface w-full max-w-md ${tint}`}
        >
            {/* Donne du relief à la teinte plate, quelle que soit la condition. */}
            <div
                aria-hidden="true"
                className="absolute inset-0 bg-linear-to-br from-white/10 via-transparent to-black/25"
            />

            <div className="relative p-7">
                <div className="flex items-center gap-4">
                    <Icon size={44} strokeWidth={1.5} className="text-gold shrink-0" aria-hidden="true" />
                    <p className="font-display text-temp text-ink">
                        {Math.round(weather.temp)}°
                    </p>
                </div>

                <h2 className="font-sans text-xl font-bold text-ink mt-4">{weather.condition}</h2>

                <div className="border-t border-white/15 mt-5 pt-5 flex items-center gap-4">
                    <p className="flex items-center gap-2 text-ink">
                        <MapPin size={18} strokeWidth={1.5} aria-hidden="true" />
                        <span>{weather.city}</span>
                    </p>

                    <button
                        onClick={onUseLocation}
                        disabled={locating}
                        className="ml-auto h-pill px-5 rounded-pill border border-white/25 text-ink text-sm font-semibold transition-colors hover:bg-white/10 disabled:opacity-60"
                    >
                        <span className="flex items-center gap-2">
                            <LocateFixed size={16} strokeWidth={1.5} aria-hidden="true" />
                            {locating ? "Localisation..." : "Utiliser ma position"}
                        </span>
                    </button>
                </div>

                {locationError && (
                    <p role="alert" className="text-sm text-ink/80 mt-3">
                        {locationError}
                    </p>
                )}
            </div>
        </section>
    );
}

export default WeatherPanel;

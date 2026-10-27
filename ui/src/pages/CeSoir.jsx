import { contextLine, greetingLines } from "../data/weather";
import { useWeather } from "../hooks/useWeather";
import { useRecommendations } from "../context/RecommendationContext";
import WeatherPanel from "../components/weather/WeatherPanel";
import MoodPicker from "../components/mood/MoodPicker";
import FeaturedMovie from "../components/movie/FeaturedMovie";
import PosterCard from "../components/movie/PosterCard";

function moodCount(count) {
    if (count === 0) return "Aucune humeur choisie";
    return count === 1 ? "1 humeur choisie" : `${count} humeurs choisies`;
}

function CeSoir() {
    const { weather, failed, locating, locationError, useMyLocation } = useWeather();
    const { selectedMoods, toggleMood, results, status, error, recommend } = useRecommendations();

    const [greeting, invitation] = greetingLines(weather);
    const [featured, ...others] = results;

    return (
        <div className="p-12 flex flex-col gap-16 max-w-7xl">
            <header className="flex flex-col lg:flex-row lg:items-center gap-10">
                <div className="flex-1">
                    <p className="text-muted">{contextLine(weather)}</p>
                    <h1 className="text-greeting mt-3">
                        {greeting}
                        <br />
                        {invitation}
                    </h1>
                </div>

                {weather ? (
                    <WeatherPanel
                        weather={weather}
                        onUseLocation={useMyLocation}
                        locating={locating}
                        locationError={locationError}
                    />
                ) : (
                    <div className="w-full max-w-md rounded-surface bg-surface p-7 text-muted">
                        {failed ? "Météo indisponible pour le moment." : "Relevé météo en cours..."}
                    </div>
                )}
            </header>

            <section>
                <MoodPicker selectedMoods={selectedMoods} onToggle={toggleMood} />

                <div className="flex flex-wrap items-center gap-4 mt-7">
                    <button
                        onClick={recommend}
                        disabled={selectedMoods.length === 0 || status === "loading"}
                        className="h-pill px-7 rounded-pill bg-gold text-on-gold font-semibold transition-opacity hover:opacity-90 disabled:opacity-40"
                    >
                        {status === "loading" ? "Recherche..." : "Trouver mon film"}
                    </button>
                    <p className="text-sm text-muted">{moodCount(selectedMoods.length)}</p>
                </div>
            </section>

            {status === "loading" && (
                <p className="text-muted">Sélection de ton film en cours...</p>
            )}

            {status === "error" && (
                <section role="alert" className="bg-surface rounded-surface p-7">
                    <p className="text-ink">{error}</p>
                    <button
                        onClick={recommend}
                        className="h-pill px-6 rounded-pill border border-edge text-ink font-semibold mt-4 transition-colors hover:bg-raised"
                    >
                        Réessayer
                    </button>
                </section>
            )}

            {status === "done" && !featured && (
                <p className="text-muted">
                    Aucun film ne ressort pour ces humeurs. Essaie une autre combinaison.
                </p>
            )}

            {status === "done" && featured && (
                <>
                    <section>
                        <h2 className="text-section mb-6">Notre choix pour ce soir</h2>
                        <FeaturedMovie movie={featured} />
                    </section>

                    {others.length > 0 && (
                        <section>
                            <h2 className="text-section mb-6">Aussi pour ta soirée</h2>
                            <div className="grid grid-cols-2 md:grid-cols-4 xl:grid-cols-6 gap-5">
                                {others.map((movie) => (
                                    <PosterCard key={movie.id} movie={movie} />
                                ))}
                            </div>
                        </section>
                    )}
                </>
            )}
        </div>
    );
}

export default CeSoir;

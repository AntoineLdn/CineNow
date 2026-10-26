import {
    Sun,
    Cloud,
    CloudFog,
    CloudRain,
    CloudDrizzle,
    Snowflake,
    CloudLightning,
    CircleHelp,
} from "lucide-react";

// Une entrée par condition émise par le weather-service (la liste fait foi dans
// shared/src/data/weather_families.toml). La direction Velours ne définit que
// quatre teintes : nuage et brouillard reprennent donc celle de la pluie.
//
// Une Map plutôt qu'un objet : les libellés du backend restent des valeurs et
// non des noms de propriétés, ce qui évite des clés mêlant ASCII et accents.
const CONDITIONS = new Map([
    [
        "Ensoleillé",
        {
            Icon: Sun,
            tint: "bg-sun",
            greeting: (city) => [`Grand soleil sur ${city}.`, "Soirée légère ?"],
        },
    ],
    [
        "Nuageux",
        {
            Icon: Cloud,
            tint: "bg-rain",
            greeting: (city) => [`Ciel couvert sur ${city}.`, "Une soirée à l'intérieur."],
        },
    ],
    [
        "Brouillard",
        {
            Icon: CloudFog,
            tint: "bg-rain",
            greeting: (city) => [`Brouillard sur ${city}.`, "Reste près de l'écran."],
        },
    ],
    [
        "Pluie",
        {
            Icon: CloudRain,
            tint: "bg-rain",
            greeting: (city) => [`Il pleut sur ${city}.`, "Reste au chaud."],
        },
    ],
    [
        "Averses",
        {
            Icon: CloudDrizzle,
            tint: "bg-rain",
            greeting: (city) => [`Des averses sur ${city}.`, "Reste au chaud."],
        },
    ],
    [
        "Neige",
        {
            Icon: Snowflake,
            tint: "bg-snow",
            greeting: (city) => [`Il neige sur ${city}.`, "Emmitoufle-toi."],
        },
    ],
    [
        "Orageux",
        {
            Icon: CloudLightning,
            tint: "bg-storm",
            greeting: (city) => [`L'orage gronde sur ${city}.`, "Baisse la lumière."],
        },
    ],
]);

const FALLBACK = {
    Icon: CircleHelp,
    tint: "bg-storm",
    greeting: (city) => [city ? `Bonne soirée, ${city}.` : "Bonne soirée.", "Choisis une humeur."],
};

/** @typedef {import("../api/client").Weather} Weather */

export function weatherLook(condition) {
    return CONDITIONS.get(condition) ?? FALLBACK;
}

/**
 * Deux lignes affichées en grand, la ville venant du backend.
 *
 * @param {Weather | null} weather
 */
export function greetingLines(weather) {
    return weatherLook(weather?.condition).greeting(weather?.city ?? "");
}

const PERIOD_LABELS = new Map([
    ["Matin", "matin"],
    ["Après-midi", "après-midi"],
    ["Soir", "soir"],
    ["Nuit", "nuit"],
]);

/**
 * « Samedi soir, Paris » : le jour vient de l'horloge locale, la période et la
 * ville du backend.
 *
 * @param {Weather | null} weather
 * @param {Date} [date]
 */
export function contextLine(weather, date = new Date()) {
    const day = date.toLocaleDateString("fr-FR", { weekday: "long" });
    const period = PERIOD_LABELS.get(weather?.period);
    const moment = period ? `${day} ${period}` : day;
    const label = moment.charAt(0).toUpperCase() + moment.slice(1);
    return weather?.city ? `${label}, ${weather.city}` : label;
}

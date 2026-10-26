import { useCallback, useEffect, useState } from "react";
import { getWeather, sendLocation } from "../api/client";

const REFRESH_MS = 30000;
// Le weather-service republie son relevé après avoir reçu la position : on lui
// laisse le temps d'arriver avant de relire.
const AFTER_LOCATION_MS = 1500;

export function useWeather() {
    const [weather, setWeather] = useState(null);
    const [failed, setFailed] = useState(false);
    const [locating, setLocating] = useState(false);
    const [locationError, setLocationError] = useState(null);

    const refresh = useCallback(async () => {
        try {
            setWeather(await getWeather());
            setFailed(false);
        } catch {
            setFailed(true);
        }
    }, []);

    useEffect(() => {
        refresh();
        const id = setInterval(refresh, REFRESH_MS);
        return () => clearInterval(id);
    }, [refresh]);

    const useMyLocation = useCallback(() => {
        if (!("geolocation" in navigator)) {
            setLocationError("La géolocalisation n'est pas disponible ici.");
            return;
        }

        setLocating(true);
        setLocationError(null);

        navigator.geolocation.getCurrentPosition(
            async ({ coords }) => {
                try {
                    await sendLocation(coords.latitude, coords.longitude);
                    setTimeout(() => {
                        refresh();
                        setLocating(false);
                    }, AFTER_LOCATION_MS);
                } catch {
                    setLocationError("La position n'a pas pu être envoyée.");
                    setLocating(false);
                }
            },
            () => {
                setLocationError("Position refusée ou indisponible.");
                setLocating(false);
            }
        );
    }, [refresh]);

    return { weather, failed, locating, locationError, useMyLocation };
}

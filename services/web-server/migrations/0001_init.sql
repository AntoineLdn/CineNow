-- Schéma du cœur de l'espace utilisateur.
-- Exécuté automatiquement au démarrage du web-server via sqlx::migrate!.

CREATE TABLE IF NOT EXISTS users (
                                     id            INTEGER PRIMARY KEY AUTOINCREMENT,
                                     username      TEXT NOT NULL UNIQUE,
                                     password_hash TEXT NOT NULL,
                                     created_at    TEXT NOT NULL DEFAULT (datetime('now'))
    );

CREATE TABLE IF NOT EXISTS liked_movies (
                                            user_id  INTEGER NOT NULL,
                                            movie_id INTEGER NOT NULL,
                                            added_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (user_id, movie_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
    );

CREATE TABLE IF NOT EXISTS watched_movies (
                                              user_id  INTEGER NOT NULL,
                                              movie_id INTEGER NOT NULL,
                                              added_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (user_id, movie_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
    );

CREATE TABLE IF NOT EXISTS favorite_genres (
                                               user_id INTEGER NOT NULL,
                                               genre   TEXT NOT NULL,
                                               PRIMARY KEY (user_id, genre),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
    );

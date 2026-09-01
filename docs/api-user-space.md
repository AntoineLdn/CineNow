# API — Espace utilisateur

Base URL : `http://localhost:3001`

L'authentification se fait par **token JWT**. Après `register` ou `login`, le
serveur renvoie un `token`. Pour toutes les routes `/me/*`, le client doit
envoyer ce token dans l'en-tête :

```
Authorization: Bearer <token>
```

Un token est valable 7 jours. Sans token valide, les routes `/me/*` renvoient
`401 Unauthorized`.

---

## Authentification (routes publiques)

### POST /auth/register
Crée un compte et renvoie un token.

Corps :
```json
{ "username": "antoine", "password": "motdepasse8+" }
```
Réponse `200` :
```json
{ "token": "eyJ...", "username": "antoine" }
```
Erreurs : `400` (username < 3 ou password < 8 caractères), `409` (username déjà pris).

### POST /auth/login
Vérifie les identifiants et renvoie un token.

Corps : identique à register.
Réponse `200` : identique à register.
Erreur : `401` (identifiants invalides — message volontairement identique que le
compte existe ou non).

---

## Espace utilisateur (routes protégées, token requis)

### Likes
| Méthode | Route | Corps | Réponse |
|---|---|---|---|
| GET | `/me/likes` | — | `200` `[550, 680, ...]` (movie_id) |
| POST | `/me/likes/:movie_id` | — | `204` |
| DELETE | `/me/likes/:movie_id` | — | `204` |

### Films vus
| Méthode | Route | Corps | Réponse |
|---|---|---|---|
| GET | `/me/watched` | — | `200` `[550, ...]` |
| POST | `/me/watched/:movie_id` | — | `204` |
| DELETE | `/me/watched/:movie_id` | — | `204` |

### Genres favoris
| Méthode | Route | Corps | Réponse |
|---|---|---|---|
| GET | `/me/favorite-genres` | — | `200` `["action", "comedie"]` |
| POST | `/me/favorite-genres` | `{ "genre": "action" }` | `204` |
| DELETE | `/me/favorite-genres/:genre` | — | `204` |

Les ajouts (POST) sont **idempotents** : ajouter deux fois le même élément ne
provoque pas d'erreur (`INSERT OR IGNORE`).

---

## Câblage côté UI

Le client `ui/src/api/auth.js` encapsule tous ces appels et gère le token en
mémoire. Exemple d'utilisation :

```js
import { login, addLike, getLikes } from "./api/auth";

await login("antoine", "motdepasse8+");   // stocke le token
await addLike(550);                        // ajoute un like (token auto-attaché)
const likes = await getLikes();            // [550]
```

Le composant `ui/src/components/AuthPanel.jsx` fournit un formulaire minimal de
connexion / inscription. Il appelle `onAuth(username)` en cas de succès, à
brancher dans l'app pour afficher l'espace utilisateur.

---

## Sécurité — état actuel et limites

**En place :**
- Mots de passe hachés avec **Argon2id** (sel unique par utilisateur, jamais en clair).
- Tokens **JWT signés** côté serveur, secret refusé s'il fait moins de 32 caractères.
- Message de login identique que le compte existe ou non (anti-énumération).
- Validation minimale des entrées (longueur username / mot de passe).

**À durcir avant une mise en production publique :**
- Servir l'API en **HTTPS** (les tokens transitent en clair en HTTP).
- **Rate-limiting** sur `/auth/login` et `/auth/register` (anti-bruteforce).
- Politique de mot de passe plus stricte (longueur max, complexité).
- **Refresh tokens** et révocation (aujourd'hui un token valide l'est 7 jours sans rappel possible).
- Rotation du `JWT_SECRET` et stockage hors `.env` (gestionnaire de secrets).
- Stockage du token côté UI dans le **store sécurisé de Tauri** plutôt qu'en mémoire volatile.

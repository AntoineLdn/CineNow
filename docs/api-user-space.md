# API — Espace utilisateur

Base URL : `http://localhost:3001`

L'authentification se fait par **token JWT**. Après `register` ou `login`, le
serveur renvoie un `token`. Pour toutes les routes protégées — `/me/*` et
`POST /recommendations` — le client doit envoyer ce token dans l'en-tête :

```
Authorization: Bearer <token>
```

Un token est valable 7 jours. Sans token valide, ces routes renvoient
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

## Recommandation (route protégée, token requis)

### POST /recommendations

Demande une recommandation et **attend son résultat** : la réponse ne revient
qu'une fois le calcul terminé, il n'y a pas de second appel à faire.

Corps :
```json
{ "moods": ["calm", "reflection"] }
```

Réponse `200` : la liste des films, dans l'ordre des scores décroissants.
```json
[{ "id": 550, "title": "Fight Club", "genres": ["drame"], "rating": 8.4, "vote_count": 30142, "poster_path": "/abc.jpg", "overview": "...", "release_date": "1999-10-15" }]
```

Un film n'est proposé que s'il remplit **les trois conditions** suivantes :

| Condition | Pourquoi |
|---|---|
| Déjà sorti (`release_date` ≤ aujourd'hui) | « Ce soir » ne doit jamais proposer un film invisible |
| `rating` non nul | sans note, rien ne permet de le classer |
| `vote_count` ≥ 50 | un film obscur noté 10/10 sur deux votes passerait devant tout le monde |

Une `release_date` absente ou illisible **exclut** le film : on ne peut pas
affirmer qu'il est sorti. Le seuil de votes est
`MIN_VOTES_FOR_RECOMMENDATION` dans `services/recommendation-service/src/lib.rs`.

Ces exclusions ne valent que pour la recommandation : `/movies` et
`/movies/by-ids` continuent de renvoyer les films à venir, que le catalogue
affiche avec un badge de sortie.

| Code | Quand |
|---|---|
| `400` | `moods` vide, ou humeur inconnue (le message la nomme) |
| `401` | token absent ou invalide |
| `503` | catalogue de films pas encore reçu du broker |
| `504` | le recommendation-service n'a pas répondu en 5 s |
| `502` | la demande n'a pas pu être publiée sur MQTT |

Les humeurs acceptées sont les sections de `shared/src/data/affinities.toml` :
`joy`, `sad`, `angry`, `calm`, `tired`, `fear`, `adventure`, `reflection`.

Sous le capot, le serveur publie la demande sur `recommendations/request` avec un
`request_id`, et attend la réponse portant le même identifiant sur
`recommendations/result`. Deux comptes qui demandent en même temps ne peuvent
donc plus recevoir le calcul l'un de l'autre.

---

## Résoudre les identifiants (route publique)

Les routes `/me/likes` et `/me/watched` ne renvoient que des `movie_id`. Pour
obtenir les films correspondants :

### GET /movies/by-ids?ids=1,2,3

Renvoie les films **dans l'ordre des ids demandés**. Ceux que le catalogue MQTT
connaît en sortent directement, les autres sont récupérés sur TMDb. Les ids
introuvables sont simplement absents du résultat, sans erreur.

| Code | Quand |
|---|---|
| `400` | `ids` absent ou vide, format invalide, ou plus de 100 ids |

---

## Forme d'un film

Toutes les routes qui renvoient des films partagent la même forme.

| Champ | Type | Remarque |
|---|---|---|
| `id` | `number` | identifiant TMDb |
| `title` | `string` | |
| `genres` | `string[]` | slugs normalisés : `"science-fiction"`, jamais `"Science-Fiction"` |
| `rating` | `number \| null` | **nul** quand `vote_count` vaut 0 |
| `vote_count` | `number` | nombre de votes TMDb |
| `poster_path` | `string \| null` | suffixe seul, l'UI reconstruit l'URL |
| `overview` | `string` | **vide** quand le film vient du flux MQTT |
| `release_date` | `string \| null` | `AAAA-MM-JJ` |

`rating` est nul et non zéro parce que TMDb renvoie `vote_average = 0` pour un
film sans vote — souvent parce qu'il n'est pas encore sorti. Le distinguer d'une
vraie note de 0 évite d'afficher « 0,0 » sur un film que personne n'a noté :
l'UI affiche « Aucune note », ou rien du tout si le film n'est pas sorti.

`GET /movie/:id` ajoute `duration`, `director` et `actors`. Son `overview` est
rempli : quand TMDb n'a pas de traduction française, la route retente en anglais.

---

## Câblage côté UI

Le client `ui/src/api/auth.js` encapsule tous les appels authentifiés et attache
le token automatiquement. Exemple d'utilisation :

```js
import { login, addLike, getLikes, getRecommendations } from "./api/auth";

await login("antoine", "motdepasse8+");   // stocke le token dans le keychain
await addLike(550);                        // token auto-attaché
const likes = await getLikes();            // [550]
const films = await getRecommendations(["calm"]);
```

Le token est confié au **gestionnaire de secrets natif de l'OS** via les
commandes Tauri `save_token` / `get_token` / `delete_token` (voir
`ui/src-tauri/src/lib.rs`) : jamais de localStorage, jamais de fichier en clair,
et la session survit au redémarrage de l'app.

Un `401` sur une route protégée purge le token et ramène l'écran de connexion.

`ui/src/components/AuthPanel.jsx` fournit le formulaire de connexion /
inscription ; c'est `AuthContext` qui porte l'état de session, et `App` affiche
l'application dès que `isLoggedIn` passe à vrai.

---

## Sécurité — état actuel et limites

**En place :**
- Mots de passe hachés avec **Argon2id** (sel unique par utilisateur, jamais en clair).
- Tokens **JWT signés** côté serveur, secret refusé s'il fait moins de 32 caractères.
- Message de login identique que le compte existe ou non (anti-énumération).
- Validation minimale des entrées (longueur username / mot de passe).
- Token stocké dans le **gestionnaire de secrets de l'OS** côté UI, jamais sur le disque en clair.
- Recommandation **authentifiée** : un compte ne peut plus recevoir le calcul demandé par un autre.

**À durcir avant une mise en production publique :**
- Servir l'API en **HTTPS** (les tokens transitent en clair en HTTP).
- **Rate-limiting** sur `/auth/login` et `/auth/register` (anti-bruteforce).
- Politique de mot de passe plus stricte (longueur max, complexité).
- **Refresh tokens** et révocation (aujourd'hui un token valide l'est 7 jours sans rappel possible).
- Rotation du `JWT_SECRET` et stockage hors `.env` (gestionnaire de secrets).

# Paysage

🇫🇷 Français · [🇬🇧 LANDSCAPE.md](https://github.com/warith-harchaoui/podcast-helper-rs/blob/master/LANDSCAPE.md)

Crates voisins puis concurrents dans l'espace « audio de podcast ou de flux vers du PCM, en Rust », comparés à `podcast-helper-rs`. Les notes
vont de ⭐ (1) à ⭐⭐⭐⭐⭐ (5), évaluées sur la tâche visée par `podcast-helper-rs` :
une URL en entrée, du PCM en sortie : classer une source (fichier local, URL audio directe, flux RSS/Atom, hôte yt-dlp, catalogue verrouillé par DRM), choisir le dernier épisode s'il s'agit d'un flux, puis la décoder par ffmpeg. Un crate optimisé pour un tout autre usage n'est pas pénalisé : la note
reflète seulement l'adéquation à *ce* créneau.

## En un coup d'œil

<!-- TABLE:START -->
| Audio de podcast en Rust | Analyse RSS / Atom | Exactitude de la durée iTunes | Classement de la source | Décodage PCM | Résolution de flux en direct | Écriture de flux | API asynchrone | Erreurs typées |
| --- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **podcast-helper-rs** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐ | ⭐ | ⭐⭐⭐⭐⭐ |
| feed-rs | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐ | ⭐ | ⭐ | ⭐ | ⭐ | ⭐⭐⭐ |
| feedparser-rs | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐ | ⭐ | ⭐ | ⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| rss | ⭐⭐⭐⭐ | ⭐⭐ | ⭐ | ⭐ | ⭐ | ⭐⭐⭐⭐⭐ | ⭐ | ⭐⭐⭐ |
| podfeed | ⭐⭐⭐ | ⭐⭐ | ⭐⭐ | ⭐ | ⭐ | ⭐⭐⭐⭐ | ⭐ | ⭐⭐ |
| ffmpeg-next | ⭐ | ⭐ | ⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐ | ⭐ | ⭐⭐ |
<!-- TABLE:END -->

## Carte de positionnement

<!-- FIGURE:START -->
Représentation 2D du tableau ci-dessus.

![Carte de positionnement](https://raw.githubusercontent.com/warith-harchaoui/podcast-helper-rs/master/assets/paysage.png)

La carte est un résumé en 2D des 8 critères : à lire comme une forme, pas comme un classement. « podcast-helper-rs » se situe dans le coin en haut à droite. Les axes se lisent **Horizontal : Ingéniosité ↔ Précision** et **Vertical : Fluidité ↔ Rigueur**.

L'axe horizontal mesure si un crate atteint seulement l'audio — le décodage PCM avec la résolution de flux en direct en sont les deux poids les plus forts — tandis que l'axe vertical mesure s'il lit correctement le flux puis le dit dans ses types, mené par l'exactitude de la durée iTunes ainsi que les erreurs typées.

Il en résulte une carte à la colonne de gauche encombrée puis deux crates seulement à droite : `podcast-helper-rs` avec `ffmpeg-next`. Tout le reste — `feed-rs`, `feedparser-rs`, `rss`, `podfeed` — s'arrête au document du flux puis vous tend une URL d'enclosure. Ce n'est pas un reproche : c'est la frontière de la catégorie, que ce crate franchit en déléguant le décodage à ffmpeg.

Ces deux crates de droite se séparent ensuite à la verticale et c'est l'enseignement à retenir : `ffmpeg-next` décode n'importe quoi sans rien connaître aux flux, tandis que ce crate analyse le flux, relit `itunes:duration` par `quick-xml` là où `feed-rs` lit mal la forme `MM:SS`, puis refuse d'emblée une page verrouillée par DRM au lieu d'échouer obscurément. `feedparser-rs` est l'analyseur pur le plus solide de la carte : c'est celui dont il faut reprendre les idées.
<!-- FIGURE:END -->

## D'où viennent les affirmations

Versions puis dates de publication lues sur l'API crates.io le 29 octobre 2026 :

| Crate | Version | Dernière publication |
|---|---|---|
| `feed-rs` | 3.0.0 | 2026-09-27 |
| `feedparser-rs` | 0.7.1 | 2026-10-21 |
| `rss` | 2.1.2 | 2026-09-10 |
| `podfeed` | 0.3.5 | 2026-08-09 |
| `ffmpeg-next` | 9.0.0 | 2026-08-05 |

Les notes de `podcast-helper-rs` lui-même proviennent de ce dépôt en version 0.1.6.

- **Ce crate obtient 4 plutôt que 5 sur l'analyse RSS/Atom et `feed-rs` obtient 5.** Ce n'est pas de la modestie : l'analyse *est* `feed-rs`. L'unique point d'écart va dans l'autre sens — `feed-rs` obtient 2 sur « Exactitude de la durée iTunes » parce qu'il lit mal `itunes:duration` sous la forme `MM:SS`, ce qui explique pourquoi `src/duration.rs` relit ce seul champ via `quick-xml`.

- **`feed-rs` 3.0.0 a une majeure d'avance sur le `feed-rs = "2"` que ce crate épingle.** Noté ici comme un fait plutôt que plaidé : l'épinglage est à jour au moment de la publication, non au moment de la lecture, puis combler cet écart est un changement à part entière avec sa propre campagne de tests.

- **Les deux notes de 1 portent sur l'écriture de flux puis sur une API asynchrone**, toutes deux absentes volontairement. `rss` obtient 5 à l'écriture : c'est le crate à choisir quand le travail consiste à *produire* un flux ; `feedparser-rs` obtient 4 sur l'asynchrone.

Laissé délibérément comme opinion plutôt que sourcé : les colonnes de style
*ergonomie*, puis l'ordre des lignes sous la référence.

## Idées à reprendre

Étudié plutôt que deviné : ce que chaque voisin fait mieux, puis ce que ce crate devrait changer pour l'égaler. Rien ici n'est un engagement : c'est la liste de lecture dont part la version suivante.

- **`feedparser-rs` — Podcast 2.0 puis JSON Feed** — Il couvre RSS 0.9x jusqu'à 2.0, Atom 0.3 avec 1.0, JSON Feed 1.0 puis 1.1, ainsi que les espaces de noms iTunes comme Podcast 2.0. `feed-rs` 2, que ce crate épingle, ne couvre ni Podcast 2.0 ni JSON Feed — or c'est dans cet espace de noms que vivent les chapitres, les transcriptions puis les étiquettes de financement.

- **`rss` — écrire un flux, pas seulement en lire un** — La colonne « Écriture de flux » de ce crate vaut 1 volontairement, mais l'asymétrie mérite d'être nommée : ce qui sait résoudre des épisodes sait généralement aussi les décrire et `rss` y obtient 5.

- **`rustypipe` — le flux RSS d'une chaîne bat la récupération de sa page** — Sa fonctionnalité `rss` existe parce que le flux est simplement plus rapide que la page. La branche `SourceKind::YtDlp` d'ici passe directement par un sous-processus ; pour une chaîne, le flux répondrait plus tôt puis sans sous-processus.

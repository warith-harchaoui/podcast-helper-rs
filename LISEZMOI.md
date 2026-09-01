# Podcast Helper (Rust)

[🇫🇷](https://github.com/warith-harchaoui/podcast-helper-rs/blob/main/LISEZMOI.md) · [🇬🇧](https://github.com/warith-harchaoui/podcast-helper-rs/blob/main/README.md)

[![License: BSD-3-Clause](https://img.shields.io/badge/License-BSD%203--Clause-blue.svg)](./LICENSE)

Réécriture en Rust de [`podcast-helper`](https://github.com/warith-harchaoui/podcast-helper). Même promesse, **URL in, PCM out** : donnez une source audio quelconque, récupérez un flux PCM (pulse-code modulation, l'échantillonnage brut non compressé attendu par les modèles de reconnaissance vocale) — pas un portage ligne à ligne du code Python, du Rust idiomatique de bout en bout.

## Périmètre v0.1

`extract_audio_stream(source: &str) -> Result<PcmStream, PodcastHelperError>` distingue en interne :

| Source | Détection | Comportement |
|---|---|---|
| Fichier local | chemin existant ou schéma `file://` | décodage direct via `ffmpeg` |
| URL audio directe (`.mp3`, `.m4a`, `.opus`, `.wav`, `.m3u8`, ou URL sans extension reconnue) | extension connue, ou repli par défaut | décodage direct via `ffmpeg` |
| Flux RSS/Atom | extension `.xml`/`.rss`/`.atom`/`.json` | parsing via [`feed-rs`](https://crates.io/crates/feed-rs), sélection automatique du dernier épisode, puis décodage de son enclosure |
| Spotify (`*.spotify.com`) | correspondance d'hôte | **refusé** : `PodcastHelperError::DrmProtected`, avec suggestion de chercher le flux RSS public de l'émission |
| Apple Podcasts (`podcasts.apple.com`) | correspondance d'hôte | **refusé**, même comportement |
| YouTube / Vimeo / SoundCloud / Twitch | correspondance d'hôte | délégué à [`youtube-helper-rs`](https://github.com/warith-harchaoui/youtube-helper-rs) : téléchargement de la piste audio dans un fichier WAV temporaire, puis décodage par le même chemin `ffmpeg` que les autres sources |
| Autre schéma (`ftp://`, ...) | — | `PodcastHelperError::UnrecognizedSource` |

Sortie : `PcmStream { sample_rate, channels, samples: Vec<f32> }`, par défaut 16 kHz mono (`ExtractOptions::default()`), configurable via `extract_audio_stream_with_options`.

```rust
let pcm = podcast_helper_rs::extract_audio_stream("https://feeds.npr.org/510289/podcast.xml")?;
println!("{} échantillons à {} Hz", pcm.samples.len(), pcm.sample_rate);
```

### Différences assumées avec l'original Python

- Pas d'itérateur asynchrone streamé image par image : v0.1 décode en un bloc (`Vec<f32>` complet). Le découpage en frames reste à faire côté appelant si besoin.
- `itunes:duration` n'est pas extrait (extension podcast non couverte par le modèle unifié de `feed-rs`) : `Episode.duration_seconds` vaut toujours `None` pour l'instant.
- Pas d'extracteur générique façon `yt-dlp` « generic » : une URL HTTP(S) qui n'est ni un flux à extension reconnue ni un hôte DRM/yt-dlp reconnu est tentée directement via `ffmpeg` (comportement volontairement permissif, car beaucoup d'URLs d'enclosure de podcasts n'ont aucune extension).

## `youtube-helper-rs` — branché

Les sources YouTube/Vimeo/SoundCloud/Twitch sont détectées (`SourceKind::YtDlp`, `src/source.rs`) et déléguées à [`youtube-helper-rs`](https://github.com/warith-harchaoui/youtube-helper-rs) (`src/ytdlp.rs`) : la piste audio est téléchargée dans un répertoire temporaire, puis le fichier WAV obtenu est décodé par le même chemin `ffmpeg` que les fichiers locaux — un seul chemin de décodage pour tous les types de source pris en charge, pas un cas particulier par plateforme.

Limite connue, héritée de `youtube-helper-rs` lui-même : `yt-dlp` peut renvoyer un `HTTP 403 Forbidden` de YouTube dans des environnements en bac à sable/de type CI sans cookies de navigateur ni fournisseur de jeton PO configuré (mesure anti-robot côté YouTube, pas un bug de l'un ou l'autre crate). L'erreur remonte proprement sous la forme `PodcastHelperError::YtDlp { .. }` dans ce cas — voir `tests/network_integration.rs`.

## Installation

Prérequis : `ffmpeg` sur le `PATH` (macOS : `brew install ffmpeg`).

```toml
[dependencies]
podcast-helper-rs = { git = "https://github.com/warith-harchaoui/podcast-helper-rs" }
```

## État du projet

- `cargo build` : OK, aucun avertissement.
- `cargo test` : **32 tests unitaires + 1 doctest passés**, 0 échec, 4 tests d'intégration réseau marqués `#[ignore]` (flux RSS réel, décodage `ffmpeg` bout-en-bout, refus Spotify, téléchargement+décodage YouTube réel via `youtube-helper-rs`) — à relancer manuellement via `cargo test -- --ignored` avant de committer un changement sur l'un de ces chemins. Le test YouTube peut échouer dans un environnement en bac à sable pour des raisons indépendantes de ce crate — voir la section ci-dessus.
- `cargo clippy --all-targets` : OK, aucun avertissement.
- **Couverture de code mesurée** (`cargo llvm-cov`, sur les tests par défaut, réseau exclu) : **92,37 % de lignes couvertes** (472 lignes, 36 non couvertes), 88,46 % de régions, 92,96 % de fonctions. Détail par fichier :

  | Fichier | Lignes couvertes |
  |---|---|
  | `pcm.rs` | 100,00 % |
  | `ffmpeg.rs` | 98,06 % |
  | `episode.rs` | 95,12 % |
  | `source.rs` | 97,01 % |
  | `ytdlp.rs` | 93,33 % |
  | `lib.rs` | 91,84 % |
  | `feed.rs` | 69,33 % (le chemin réseau `fetch_feed`/`latest_episode` n'est exercé que par les tests `#[ignore]`, donc absent de cette mesure) |

  Pour relancer la mesure :

  ```bash
  # une seule fois : outils LLVM (ce projet utilise Homebrew LLVM, pas rustup)
  export LLVM_COV=/opt/homebrew/opt/llvm/bin/llvm-cov
  export LLVM_PROFDATA=/opt/homebrew/opt/llvm/bin/llvm-profdata

  cargo llvm-cov --summary-only        # résumé texte
  cargo llvm-cov --html                # rapport HTML dans target/llvm-cov/html/index.html
  ```

## Projets liés

Fait partie du même socle d'outils locaux que [`podcast-helper`](https://github.com/warith-harchaoui/podcast-helper) (Python) et la suite [AI Helpers](https://github.com/warith-harchaoui/ai-helpers). Réécriture indépendante, pas une liaison (*binding*).

## Licence

BSD-3-Clause, voir [LICENSE](LICENSE).

## Auteur

[Warith HARCHAOUI](https://linkedin.com/in/warith-harchaoui)

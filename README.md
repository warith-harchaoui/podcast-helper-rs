# podcast-helper-rs

Port Rust de [`podcast-helper`](https://github.com/warith-harchaoui/podcast-helper) (Python, même auteur). Même promesse, **URL in, PCM out** : donnez une source audio quelconque, récupérez un flux PCM (pulse-code modulation, l'échantillonnage brut non compressé attendu par les modèles de reconnaissance vocale) — sans réécrire ligne à ligne le code Python, en Rust idiomatique.

Ce crate est destiné à être consommé par [`scribe-reunion`](https://github.com/warith-harchaoui/scribe-reunion) (workspace Cargo, architecture ports/adaptateurs) comme adaptateur d'entrée audio.

## Périmètre v0.1

`extract_audio_stream(source: &str) -> Result<PcmStream, PodcastHelperError>` distingue en interne :

| Source | Détection | Comportement |
|---|---|---|
| Fichier local | chemin existant ou schéma `file://` | décodage direct via `ffmpeg` |
| URL audio directe (`.mp3`, `.m4a`, `.opus`, `.wav`, `.m3u8`, ou URL sans extension reconnue) | extension connue, ou repli par défaut | décodage direct via `ffmpeg` |
| Flux RSS/Atom | extension `.xml`/`.rss`/`.atom`/`.json` | parsing via [`feed-rs`](https://crates.io/crates/feed-rs), sélection automatique du dernier épisode, puis décodage de son enclosure |
| Spotify (`*.spotify.com`) | correspondance d'hôte | **refusé** : `PodcastHelperError::DrmProtected`, avec suggestion de chercher le flux RSS public de l'émission |
| Apple Podcasts (`podcasts.apple.com`) | correspondance d'hôte | **refusé**, même comportement |
| YouTube / Vimeo / SoundCloud / Twitch | correspondance d'hôte | détecté mais **non implémenté** : `PodcastHelperError::YtDlpNotImplemented` (voir ci-dessous) |
| Autre schéma (`ftp://`, ...) | — | `PodcastHelperError::UnrecognizedSource` |

Sortie : `PcmStream { sample_rate, channels, samples: Vec<f32> }`, par défaut 16 kHz mono (`ExtractOptions::default()`), configurable via `extract_audio_stream_with_options`.

```rust
let pcm = podcast_helper_rs::extract_audio_stream("https://feeds.npr.org/510289/podcast.xml")?;
println!("{} échantillons à {} Hz", pcm.samples.len(), pcm.sample_rate);
```

### Différences assumées avec le Python

- Pas d'itérateur asynchrone streamé image par image : v0.1 décode en un bloc (`Vec<f32>` complet). Le découpage en frames reste à faire côté appelant si besoin.
- `itunes:duration` n'est pas extrait (extension podcast non couverte par le modèle unifié de `feed-rs`) : `Episode.duration_seconds` vaut toujours `None` pour l'instant.
- Pas d'extracteur générique façon `yt-dlp` "generic" : une URL HTTP(S) qui n'est ni un flux (extension connue) ni un hôte DRM/yt-dlp reconnu est tentée directement via `ffmpeg` (comportement volontairement permissif, car beaucoup d'URLs d'enclosure de podcasts n'ont aucune extension).

## `youtube-helper-rs` — pas encore branché

Les sources YouTube/Vimeo/SoundCloud/Twitch sont détectées (`SourceKind::YtDlp`) mais renvoient `PodcastHelperError::YtDlpNotImplemented` : au moment où ce crate a été écrit, [`youtube-helper-rs`](https://github.com/warith-harchaoui/youtube-helper-rs) n'était pas encore publié sur GitHub. Le point de branchement est marqué `// TODO(youtube-helper-rs)` dans `src/error.rs` et `src/lib.rs`. Suivi : voir l'issue [#1](https://github.com/warith-harchaoui/podcast-helper-rs/issues/1) sur ce dépôt.

## Installation

Prérequis : `ffmpeg` sur le `PATH` (macOS : `brew install ffmpeg`).

```toml
[dependencies]
podcast-helper-rs = { git = "https://github.com/warith-harchaoui/podcast-helper-rs" }
```

## État du projet

- `cargo build` : OK, aucun avertissement.
- `cargo test` : **32 tests unitaires + 1 doctest passés**, 0 échec, 3 tests d'intégration réseau marqués `#[ignore]` (feed RSS réel, décodage `ffmpeg` bout-en-bout, refus Spotify) — vérifiés manuellement une fois via `cargo test -- --ignored` (les 3 passent) avant de committer, pas exécutés par défaut.
- `cargo clippy --all-targets` : OK, aucun avertissement.
- **Couverture de code mesurée** (`cargo llvm-cov`, sur les tests par défaut, réseau exclu) : **92.24 % de lignes couvertes** (451 lignes, 35 non couvertes), 88.82 % de régions, 92.75 % de fonctions. Détail par fichier :

  | Fichier | Lignes couvertes |
  |---|---|
  | `pcm.rs` | 100.00 % |
  | `ffmpeg.rs` | 98.06 % |
  | `episode.rs` | 95.12 % |
  | `source.rs` | 97.01 % |
  | `lib.rs` | 90.70 % |
  | `feed.rs` | 69.33 % (le chemin réseau `fetch_feed`/`latest_episode` n'est exercé que par les tests `#[ignore]`, donc absent de cette mesure) |

  Pour relancer la mesure :

  ```bash
  # une seule fois : llvm-tools (via Homebrew LLVM, ce projet n'utilise pas rustup)
  export LLVM_COV=/opt/homebrew/opt/llvm/bin/llvm-cov
  export LLVM_PROFDATA=/opt/homebrew/opt/llvm/bin/llvm-profdata

  cargo llvm-cov --summary-only        # résumé texte
  cargo llvm-cov --html                # rapport HTML dans target/llvm-cov/html/index.html
  ```

## Licence

BSD-3-Clause, voir [LICENSE](LICENSE).

## Auteur

[Warith HARCHAOUI](https://linkedin.com/in/warith-harchaoui)

# podcast-helper-rs as an agent skill

`skills/podcast-helper-rs/` packages this crate as a **Claude Skill** *and* an
**OpenCode skill** — both ecosystems read the same `SKILL.md` (YAML frontmatter
+ Markdown body + progressive-disclosure `references/`). Installing it lets an
agent discover the crate and parse podcast feeds, resolve episodes and decode them to PCM on the user's behalf, without the user
opening a terminal.

## Layout

```
skills/podcast-helper-rs/
├── SKILL.md                  # name + trigger-rich description + instructions
└── references/
    ├── cli-reference.md      # every subcommand, flag and exit code
    ├── surfaces.md           # which surfaces exist here, and which do not
    └── triggers.md           # exhaustive, auditable trigger catalogue
```

Progressive disclosure, as the published skill standard prescribes: the YAML
frontmatter is always in the host model's context, `SKILL.md`'s body loads only
when the skill looks relevant, and `references/*.md` load only when a task
actually needs that depth. The `description` stays under the 1024-character cap;
the exhaustive trigger list lives in `references/triggers.md`.

## Install for Claude Code / Claude Desktop

Skills live under `~/.claude/skills/` (user) or `.claude/skills/` (project). To
track this repo's copy rather than duplicate it, symlink it:

```bash
ln -sfn "$PWD/skills/podcast-helper-rs" ~/.claude/skills/podcast-helper-rs
# per-project instead:
mkdir -p /path/to/project/.claude/skills
ln -sfn "$PWD/skills/podcast-helper-rs" /path/to/project/.claude/skills/podcast-helper-rs
```

## Install for OpenCode

OpenCode reads skills from `~/.opencode/skills/` (or `~/.config/opencode/skills/`):

```bash
mkdir -p ~/.opencode/skills
ln -sfn "$PWD/skills/podcast-helper-rs" ~/.opencode/skills/podcast-helper-rs
```

## Keeping triggers enforced

The host model only sees `SKILL.md`'s `description` before deciding to load the
skill, so every real trigger must be represented there, compressed into the
1024-character budget. `references/triggers.md` is the human-reviewable
superset and mirrors the repo-root `TRIGGERS.md`. When one changes, change the
other — `tests/skill.rs` fails `cargo test`, and therefore the pre-push hook,
if they drift apart.

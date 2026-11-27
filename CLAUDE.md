# Rules for AI agents working in this repository

1. Read [SAFETY.md](SAFETY.md) and [DECISIONS.md](DECISIONS.md) first. DECISIONS.md is the source of truth; if you change a design choice, record it there in the same change.
2. Never write instructions that would lead someone to build or power a circuit above 50 volts until the lab safety policy exists (D-10). Design files and simulation are fine; "build this and plug it in" is not.
3. Never describe anything here as safe or ready for use on a person or animal.
4. Label simulated results as simulated, and unverified facts as unverified. Give the source for every number.
5. Firmware is Rust (D-08). No new language dependencies without an owner decision.
6. Write documents in plain language. The owner is not a programmer or an electrical engineer; explain decisions and their reasons, not implementation detail.
7. The repository is public (owner decision, 27 November 2026). Never commit patient data, anything under `data/`, personal contact details of third parties, or secrets. Buildable high-voltage design files (schematics, board layouts) are not published until the patent review is done (D-13). Do not add collaborators or post about the project elsewhere without the owner asking.
8. Research notes live in `research_notes/`, one file per topic. Do not overwrite another topic's file.

# Rules for AI agents working in this repository

1. Read [SAFETY.md](SAFETY.md) and [DECISIONS.md](DECISIONS.md) first. DECISIONS.md is the source of truth; if you change a design choice, record it there in the same change.
2. Never write instructions that would lead someone to build or power a circuit above 50 volts until the lab safety policy exists (D-10). Circuit diagrams, parts lists and simulations are fine and are published (D-13); every circuit file carries the words "NOT FOR CONSTRUCTION — unreviewed design" until a qualified engineer has reviewed it. Assembly steps, bring-up procedures and "build this and plug it in" are not written.
3. Never describe anything here as safe or ready for use on a person or animal.
4. Label simulated results as simulated, and unverified facts as unverified. Give the source for every number.
5. Firmware is Rust (D-08). No new language dependencies without an owner decision.
6. Write documents in plain language. The owner is not a programmer or an electrical engineer; explain decisions and their reasons, not implementation detail.
7. The repository is public (owner decision, 27 November 2026), including circuit designs (D-13). Never commit patient data, anything under `data/`, personal contact details of third parties, or secrets. Do not add collaborators or post about the project elsewhere without the owner asking.
9. Circuit diagrams are generated from pin tables in `hardware/<board>/specs/` by `tools/schgen` (D-58). The pin tables are the source of truth; never edit a generated `.kicad_sch` by hand. Run `bash tools/build.sh` after any change.
8. Research notes live in `research_notes/`, one file per topic. Do not overwrite another topic's file.

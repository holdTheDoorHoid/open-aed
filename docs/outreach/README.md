# Outreach for real cardiac-arrest recordings

Drafted 27 November 2026. **These are drafts for the project owner to edit and send. Nothing has been sent to anyone.**

## Why

The shock-decision software has gone as far as public data can take it (see [FINDINGS.md](../FINDINGS.md), F-10 to F-12). Reaching the target needs recordings of real cardiac arrests taken through defibrillator pads, from many patients. Research on 27 November 2026 found that nobody offers such a dataset openly; every holder shares only through a collaboration. The details and sources are in `research_notes/Open source belt worn AED/ohca_data_sources.md`.

## The honest position

This project is led by an individual with no institution and no ethics board. Most data holders need:

- a person at an institution who can sign a data agreement, and
- a letter from an ethics board (an "IRB") approving the work, or stating that work on de-identified recordings does not need approval. Federal guidance says the institution, not the researcher, should make that call.

So the realistic routes are, in order:

1. **A university partner.** A researcher who already holds or can obtain the data, with the project supplying the open software. This is the main aim of letters 1 and 2.
2. **"Run our code on your data."** The holder keeps the recordings and runs the project's published evaluation tool on them, sending back only the scores. No data changes hands. Letters 2 and 3 offer this.
3. **A non-profit with an independent ethics board.** Slower and costs money. Not pursued yet.

## Who to write to, in order

| Order | Who | Letter | What to ask | Contact route | Check before sending |
|---|---|---|---|---|---|
| 1 | University of Pennsylvania Center for Resuscitation Science (Philadelphia) | [01](01-local-university-partner.md) | A conversation about collaborating | The centre's general address, listed on its website "for general questions": crsinfo@uphs.upenn.edu | The centre's pages disagree about who directs it. Address the centre, not a person. Confirm the address is still current. |
| 2 | Authors of the "Cardially" dataset (University of Brescia) | [04](04-licence-confirmation.md) | Written confirmation of the licence | The corresponding author named on the dataset page | The dataset page says one licence and a summary of the article says another. Find the corresponding author's address on the dataset page. |
| 3 | Research groups holding pad recordings: Seattle / King County (University of Washington); Dallas–Fort Worth with the University of the Basque Country; Paris Fire Brigade with Inserm | [02](02-research-group.md) | Collaboration, or running our tool on their data | The corresponding author printed on a recent paper from each group | Read each group's most recent paper first and name it in the letter. Their data-sharing statements could not be opened during research. |
| 4 | Defibrillator makers with published research use of device logs | [03](03-manufacturer.md) | Whether a research data route exists | The company's medical or clinical affairs contact | No maker was found to have a published data-request programme. Expect "no" or silence. |

Two open datasets can be used straight away, with limits: "Cardially" (260 recordings of fibrillation only, apparently digitised from paper, licence to be confirmed) and a Paris Fire Brigade set of pad-resistance recordings during CPR (no heart signal). Neither needs a letter to download; the first needs letter 04 before the project relies on it.

Not worth writing to yet: the large US trial archive (NHLBI BioLINCC) and the CARES registry. Both appear to hold tables of outcomes, not recordings, and both require an institution.

## What to attach to every letter

- [00-project-summary.md](00-project-summary.md): one page on what the project is.
- [05-data-handling.md](05-data-handling.md): what the project promises to do with any data.

## Before sending anything

- Decide the name and contact details that go at the bottom. The letters say `[Your name]`.
- Decide whether the repository is public by then. The letters assume it is, because "open" is the main thing the project has to offer; a holder cannot inspect a private repository.
- Have someone outside the project read letter 01. A clinician or researcher friend will spot wording that sounds naive to that audience.

## Log

| Date | Sent to | Letter | Reply |
|---|---|---|---|
| | | | |

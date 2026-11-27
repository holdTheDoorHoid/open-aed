# Letter 1: local university partner

**To:** University of Pennsylvania Center for Resuscitation Science (general enquiries address)
**Subject:** Open source defibrillator rhythm analysis: request for a short conversation

---

Dear colleagues at the Center for Resuscitation Science,

I lead a small open source project in Pennsylvania that is designing a belt-worn defibrillator for trained responders, and publishing everything: designs, firmware, test results and failures. It is at the simulation stage. It is not a medical device and nothing has been built or used on anyone.

I am writing because we have reached the limit of what public data can tell us, and your centre is the nearest group with real expertise in this.

Our rhythm analysis leaves normal rhythms alone 99.8% of the time on the public PhysioNet databases, but detects only about 71% of coarse ventricular fibrillation against the 90% target, on patients held back from tuning. Those databases are monitor recordings from a few dozen patients. They have no compression artifact, no children, and nothing recorded through pads during an out-of-hospital arrest. We do not think more tuning on them would be honest.

I would be grateful for twenty minutes of someone's time to ask three things:

1. Whether anyone at the centre would be interested in collaborating, with the centre holding any data and the project contributing open, tested software and all results.
2. If sharing recordings is not possible, whether the centre could run our published scoring tool on recordings it already holds and return only the summary scores. No data would leave your systems.
3. Two clinical questions our simulation has raised about fully automatic devices, on which we need a clinician's view: how strictly a shock should be cancelled if the rhythm analysis wavers during the countdown, and how to treat a rhythm that changes in the last two seconds before delivery.

I should be plain about our position. I have no institutional affiliation and no ethics board. I understand that work with patient recordings would need to sit under your institution's review and agreements, and I am not asking you to bypass any of that.

A one-page summary and our data-handling commitments are attached. The repository, including the evaluation report and our list of known problems, is at https://github.com/holdTheDoorHoid/open-aed.

Thank you for considering it.

[Your name]
[Email and telephone]

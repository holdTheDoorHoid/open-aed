# Letter 2: research groups that hold pad recordings

**To:** the corresponding author of a recent paper from the group
**Subject:** Open source defibrillator rhythm analysis: possible collaboration on [name of their dataset]

*Before sending: read the group's most recent paper and fill in the two bracketed places. A letter that shows you read their work gets answered far more often.*

---

Dear Dr [surname],

I read your paper [title, journal, year] with great interest, in particular [one specific thing: for example, the use of pad impedance to mark compression periods].

I lead a small open source project designing a belt-worn defibrillator for trained responders. Everything is published: designs, firmware, test results and failures. It is at the simulation stage, is not a medical device, and has not been used on anyone.

Our rhythm analysis has been scored on the public PhysioNet databases with patients held back for testing. It leaves normal rhythms alone 99.8% of the time but detects only about 71% of coarse ventricular fibrillation. We have concluded that public monitor recordings cannot take it further: they have no compression artifact, no pad impedance, and no out-of-hospital arrests.

Your recordings are exactly what the public databases lack. I would like to ask whether either of the following might be possible:

1. **You run our tool.** Our scoring tool is open source and has no dependencies beyond the Rust compiler. If your group ran it on your recordings and returned the summary tables, no data would leave your institution, and you would be free to publish or discard the result. We would adapt the tool to your file format.
2. **A collaboration** under your institution's agreements and ethics review, with your group holding the data and ours contributing the software.

I am not affiliated with an institution and have no ethics board, so I am not in a position to sign a data agreement myself, and I am not asking for recordings to be sent to me.

One question I would ask in either case: whether algorithm settings or model parameters derived from your data could be published under an open source licence. If the answer is no, the first option still gives us an honest external score, which is what we need most.

A one-page summary and our data-handling commitments are attached. The repository is at https://github.com/holdTheDoorHoid/open-aed.

With thanks for your time,

[Your name]
[Email]

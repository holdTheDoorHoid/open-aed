# Open AED: one-page summary

**What it is.** An open source design for a small defibrillator that a police officer, paramedic or medical professional can carry on a belt, with one set of pads for adults and children. Hardware designs, firmware and research are published under share-alike licences.

**What it is not.** It is not a medical device, is not approved by any regulator, and has never been used on a person or animal. No hardware has been built. The project does not make or distribute devices or kits. Its aim is a documented, tested reference design that a regulated manufacturer could take through approval.

**Where it stands (November 2026).**

- A research review of compact defibrillators, waveforms, rhythm analysis and the regulatory path.
- Firmware in Rust for rhythm analysis, shock planning and the rescue sequence, running in simulation only, with about 300 automated tests.
- A published tool that scores the rhythm analysis against annotated recordings.

**What the rhythm analysis scores today.** On the public PhysioNet databases, with patients held back for testing: about 71% of coarse ventricular fibrillation detected against the American Heart Association target of 90%, and normal rhythm correctly left alone 99.8% of the time. The project publishes its failures as well as its successes; the full report and the list of known problems are in the repository.

**Why it needs help.** Public databases are hospital and monitor recordings from a few dozen patients. They contain no chest-compression movement, no children, and no recordings taken through defibrillator pads during real cardiac arrests. The project has concluded that it cannot honestly improve or validate its analysis without such recordings.

**Who is behind it.** A volunteer project led by an individual in Pennsylvania, with no institutional affiliation and no commercial interest.

**Where to look.** `https://github.com/holdTheDoorHoid/open-aed`

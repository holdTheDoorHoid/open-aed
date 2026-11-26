# Safety

These rules apply to everyone who contributes, human or AI agent. They are not negotiable inside a pull request; changing them needs a recorded decision by the project owner in [DECISIONS.md](DECISIONS.md).

## 1. This is not a medical device

- Nothing built from this repository may be connected to a person or an animal.
- In the United States an AED, its pads and its battery are all in the FDA's strictest class (Class III) and need premarket approval. The EU classes them the same way. This project holds no approval of any kind.
- Do not describe anything built here as an "AED" to someone who might rely on it. Until a regulated manufacturer holds an approval, it is a research and education platform.
- Do not carry a prototype as if it were a working defibrillator. A device that looks ready but fails is worse than no device, because it delays the real one.

## 2. The energy can kill the builder

A defibrillator stores roughly 150 to 250 joules at about 2,000 volts. That is far above the level that stops a heart. It stays in the capacitor after the power is off.

Until a written lab safety policy exists (Phase 2 of the [plan](docs/PLAN.md)), **no one builds, powers or probes any circuit above 50 volts for this project.**

For scale: the US electrical safety standard (NFPA 70E) treats a capacitor as hazardous above 0.25 joules once it is at 400 volts or more. This device's capacitor holds roughly 800 times that. Capacitors can also recover part of their charge minutes after being discharged; for the camera-flash type that can be about 2 joules, which is still dangerous.

The lab safety policy must cover, at minimum:

- Who is qualified to do high-voltage work, and who reviews their setup.
- Never working alone.
- An enclosed, interlocked test fixture; no open bench work on a charged capacitor.
- A discharge circuit that empties every storage capacitor by default, a permanent bleeder resistor as backup, a separate resistor-type discharge tool, and a "prove it is dead" check before touching.
- Shorting straps left in place on any capacitor that is not in use, as visible proof it is safe and to stop charge creeping back.
- Shocks delivered only into a rated test load, never into anything else.
- What to do in an emergency, including where the nearest real AED is.

## 3. Order of work

The hazard-free work comes first on purpose:

1. Research, requirements, simulation.
2. Low-voltage electronics and software: heart-rhythm sensing and analysis, tested against recorded data.
3. Test equipment: a test load and a way to measure delivered energy.
4. Only then, high-voltage circuits, on a large open bench layout inside an interlocked enclosure.
5. Only after that works, shrinking it.

## 4. Software that decides to shock

- The shock decision is the most safety-critical software in the device. It is developed to the practices of IEC 62304 Class C from the first line, not retrofitted.
- No single fault, hardware or software, may cause a shock that was not advised or prevent one that was. The design uses an independent second check on the shock path.
- Claims about detection accuracy must state the data they were measured on. Public heart-rhythm databases flatter an algorithm compared with real cardiac-arrest recordings.

## 5. Honest reporting

- Every claim of "works" names what was tested, on what, and what was not tested.
- Simulated results are labelled as simulated.
- Research notes mark anything that was not checked against a source.

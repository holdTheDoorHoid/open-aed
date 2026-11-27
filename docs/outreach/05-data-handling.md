# Data-handling commitments

What the Open AED project undertakes for any patient recordings it is allowed to work with. These are offered as a starting point; a data holder's own agreement takes precedence.

**What we ask for, and nothing more**

- Heart signal from the defibrillator pads, and pad impedance if recorded.
- Rhythm annotations and the times of shocks and chest-compression periods.
- Patient age band (adult or child) only.

**What we ask holders to remove before anything is shared**

- All dates and clock times. Times relative to the start of each recording are enough.
- Device serial numbers and any device identifier.
- Scene audio and any voice recording.
- Case numbers, incident numbers, locations and any other unique code.

**What we will not do**

- Attempt to identify any person.
- Redistribute recordings, or publish any recording or excerpt, without written permission.
- Put recordings in the public repository, or on any service the holder has not agreed to.
- Use the recordings for anything other than developing and validating rhythm analysis.

**What we will do**

- Keep recordings on encrypted storage, with access limited to named people.
- Delete them on request or when the agreement ends, and confirm in writing.
- Publish methods and summary results, crediting the source as the holder wishes, and show the holder a draft first.
- Ask in writing, before publishing, whether any algorithm settings or model parameters derived from the recordings may be released under an open source licence, and abide by the answer.

**Our preferred arrangement**

The holder keeps the recordings and runs our published evaluation tool on them, returning only summary scores. Under that arrangement no patient data leaves the holder's systems at all.

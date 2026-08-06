# linienschluessel

A measured spectrum holds hundreds to thousands of lines and assigning them to level transitions is handwork, with documented failures such as the J quantum numbers in Tm II that had to be revised after being assigned wrongly. Formally it is clean: given observed wavelengths with intensities and predicted levels, find the assignment satisfying the Ritz combination principle, respecting the selection rules and best explaining the intensity ratios, returned with uncertainties rather than as a single answer. NIST supplies levels and lines separately, so the method is validated against already-solved spectra before being turned loose on the unidentified lines in solar, fusion and laboratory spectra.

Planning happens on the issue tracker first. Every decision that shapes
the architecture is written down there with its reasons before the code
that depends on it exists.

See [NOTICE.md](NOTICE.md) for the intended-use notice.

# synfx-dsp

synfx-dsp Real Time Audio Synthesis and Effect DSP algorithms and utility functions for Rust.

Most of the algorithms and implementations in this library have been
implemented for [HexoDSP](https://github.com/WeirdConstructor/HexoDSP) and used
in [HexoSynth](https://github.com/WeirdConstructor/HexoSynth). I factored them out, because
they seem useful in other contexts too, for instance the [synfx-dsp-jit](https://github.com/WeirdConstructor/synfx-dsp-jit)
crate.

I collected most of the algorithms in this crate from various GPLv3 compatible
sources. They also were partially translated from multiple different C++ projects.
I tried to document the source and source license diligently in the comments of this crate.
I apologize if any attribution is missing and would welcome patches or reports.

Feel free to use these algorithms and utilities. Help, patches and additions are appreciated
if they comply with the GPL-3.0-or-later license and don't break the test suite in HexoDSP.

**Attention:** HexoDSP comes with a large test suite that also covers these algorithms. And that is the one
that also has to pass if these algorithms are touched. The flip side is, that these implementations
are actually covered by a test suite.


License: GPL-3.0-or-later

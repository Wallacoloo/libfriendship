# libfriendship
A dsp library for rendering sounds using additive synthesis in real-time

**Note**: libfriendship is far from stable, and not at all useful as of yet.
Give it time.

# Purpose
libfriendship exposes an interface for building complex sounds by constructing
a tree of effects through which the user passes individual frequencies.

For example, one might start with a simple sine wave at 440 Hz (A4) and then:

1. Send it through an effect that generates harmonics at different amplitudes.
2. Send that through an effect that individually detunes each frequency component.
3. Apply a volume envelope to create a "twang" sound
4. Send it through a delay effect that creates temporal copies of the sound

Throughout this process, the sound is passed around internally in its frequency
representation. This allows many effects (e.g. filtering, equalization) to be
implemented very trivially and encourages more unique effects that are
difficult to achieve in the time-domain (like detuning each harmonic individually).
It also moves most of the computationally-intensive portions of audio synthesis
into a single location so that anyone extending the library doesn't need to
worry as much about ruthleslly optimizing their code.

# Design goals
libfriendship aims to be *safe*, *easy to use/understand* and *versatile*
(orderered by priority). It will remain limited only to sound synthesis, which
means it won't provide any interfaces for exporting the output to a file or
handling audio playback - there are other libraries for those tasks.

# License
libfriendship is published under the MIT license (a fairly liberal license).
If distributed in a project licensed under something more restrictive, like the
GPL, I ask that you make it clear to your developers that the libfriendship
code is available under the MIT license (for example, in the license portion
of your readme, mention "this code is licensed under [x], with the following
exceptions: lib/libfriendship (MIT License)")
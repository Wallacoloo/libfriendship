# libfriendship
A dsp library for rendering sounds in a Direct Form system in real-time

**Note**: libfriendship is far from stable, and not at all useful as of yet.
Give it time.

# Purpose
libfriendship exposes an interface for building complex sounds by constructing
a tree of effects through which the user passes signals.

For example, one might start with a simple sine wave at 440 Hz (A4) and then:

1. Send it through an effect that generates harmonics at different amplitudes.
2. Send that through an effect that individually detunes each frequency component.
3. Apply a volume envelope to create a "twang" sound.
4. Send it through a delay effect that creates temporal copies of the sound.

Importantly, all the audio processing occurs in one isolated block of hot
library code. This allows for solutions to the security, performance and
architecture/OS support limitations associated with binary audio plugin formats.

# Design goals
libfriendship aims to be *safe*, *easy to use/understand* and *versatile*
(orderered by priority). It will remain limited only to sound synthesis, which
means it won't provide any interfaces for exporting the output to a file or
handling audio playback - there are other libraries for those tasks.

# License
libfriendship is published under the MIT license.

# Liberty 5 Pro (D1203), experimental support

This implementation connects directly over RFCOMM. It currently reads battery
levels, firmware, the TWS connection state, and the existing button assignments.
The Device Events field shows recent command IDs and payload lengths. It does
not display or record microphone audio.

Arbitrary Android app/deep-link actions are not implemented yet. First verify
which command a physical press sends with the official Soundcore app stopped.
The Anka start command observed in the vendor parser is `18:03`, but its arrival
from this model has not yet been demonstrated. Ordinary playback and volume
gestures may be handled inside the earbuds or by Android Bluetooth profiles.

The state parser is based on the public capture in
[issue #342](https://github.com/Oppzippy/OpenSCQ30/issues/342), with identifiers
redacted in the fixture. The related
[Pro Max proposal](https://github.com/Oppzippy/OpenSCQ30/pull/305) documents the
TLV layout. Unknown tags are skipped; malformed packets and other model IDs are
rejected. Settings remain read-only until their write commands are verified.

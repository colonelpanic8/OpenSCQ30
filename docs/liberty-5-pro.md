# Liberty 5 Pro (D1203), experimental support

This implementation connects directly over RFCOMM. It currently reads battery
levels, firmware, the TWS connection state, and the existing button assignments.
The Device Events field shows recent command IDs and payload lengths. It does
not display or record microphone audio.

The integration branch includes the Android custom-action settings described in
[Custom Android actions](custom-actions.md). Anka command `18:03` is forwarded
as an assistant request. Its start/stop payload is not interpreted: either kind
of request opens the configured link when custom actions are enabled. No
recording commands are sent and audio packets do not trigger actions.

This wiring is experimental. The command is identified by the vendor dispatcher,
but delivery from a physical D1203 gesture has not yet been observed. Test the
link using the settings Test action button, then verify a physical Anka press
with the official Soundcore app stopped. Ordinary playback and volume gestures
may be handled inside the earbuds or by Android Bluetooth profiles.

The state parser is based on the public capture in
[issue #342](https://github.com/Oppzippy/OpenSCQ30/issues/342), with identifiers
redacted in the fixture. The related
[Pro Max proposal](https://github.com/Oppzippy/OpenSCQ30/pull/305) documents the
TLV layout. Unknown tags are skipped; malformed packets and other model IDs are
rejected. Settings remain read-only until their write commands are verified.

## Sound-mode action workaround

Physical hardware captures contain `06:01` sound-mode reports and `0b:02`
audio-focus reports. The latter is not an assistant or button-press event.
The Android Sound mode update trigger uses distinct seven-byte `06:01` reports
as an alternative to the unverified Anka trigger. No firmware setting is written
by this workaround. The normal sound-mode change still occurs, and changes from
other apps or automatic adjustments can also launch the action. Identical
consecutive reports are suppressed, with a separate one-second launch debounce.
The first report after each connection can trigger an action. See the custom
Android actions page for configuration.

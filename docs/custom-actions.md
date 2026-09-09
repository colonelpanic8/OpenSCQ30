# Custom Android actions

In Settings, choose a trigger, enter an app link or URL, optionally restrict it
to an app package, and save the action. Test action opens the current fields
immediately. Enable custom earbud action to use the saved trigger and target.
For example, `paseo://live-voice` with package `sh.paseo.assembly` opens Paseo Live
Voice when that build is installed.

Available triggers:

- **Assistant request**: the experimental Liberty 5 Pro integration forwards
  command `18:03`. Delivery from a physical Anka gesture is still unverified.
- **Sound mode update (experimental)**: the Liberty 5 Pro forwards distinct,
  seven-byte `06:01` sound-mode reports. Identical consecutive reports are
  suppressed. The first report after connecting is eligible to trigger, even
  without a previous report to compare. A sound-mode gesture still changes the
  sound mode; this does not replace its firmware action. Settings changes from
  another app or automatic adjustments may also trigger. This is a state-based
  workaround, not a raw button-press event.

Existing saved configurations retain Assistant request as their trigger.

Grant Display over other apps through the settings button to allow opening the
chosen app from the background. No overlay is drawn. Without this permission,
device events do not launch anything; Test action still works from the screen.
See [Android background activity launch rules](https://developer.android.com/guide/components/activities/secure-bal).

The event API has no replay and drops lagged events. Repeated events within one
second are suppressed. Disconnecting cancels collection. Targets are disabled by
default; incoming events never carry URLs or intent payloads. Microphone capture
is not part of this feature.

A device driver must implement `subscribe_to_events()` and emit the appropriate
`DeviceEvent` variant. Drivers returning the default `None` do not trigger
custom actions. Assistant request forwards both start and stop requests because
their payload semantics have not been verified. Audio packets are ignored.

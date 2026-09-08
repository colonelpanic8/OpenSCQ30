# Custom Android actions

In Settings, enter an app link or URL, optionally restrict it to an app package,
and save the action. Test action opens the current fields immediately. Enable
custom assistant action to use the saved target for future assistant events.
For example, `paseo://live-voice` with package `sh.paseo.assembly` opens Paseo Live
Voice when that build is installed.

Grant Display over other apps through the settings button to allow opening the
chosen app from the background. No overlay is drawn. Without this permission,
physical events do not launch anything; Test action still works from the screen.
See [Android background activity launch rules](https://developer.android.com/guide/components/activities/secure-bal).

The event API is separate from state notifications, has no replay, and drops
lagged events. Repeated events within one second are suppressed. Disconnecting
cancels collection. Targets are disabled by default; incoming events never carry
URLs or intent payloads. Microphone capture is not part of this feature.

A device driver must implement `subscribe_to_events()` and emit
`DeviceEvent::AssistantRequested` for an assistant request. Drivers that
return the default `None` do not trigger custom actions. The Liberty 5 Pro
integration forwards Anka command `18:03`, including both start and stop requests,
as this event. Physical delivery is still unverified.
It does not turn every firmware gesture into an app event.

package com.oppzippy.openscq30.features.soundcoredevice.service

import com.oppzippy.openscq30.lib.bindings.ConnectionStatusCallback
import com.oppzippy.openscq30.lib.bindings.DeviceEventCallback
import com.oppzippy.openscq30.lib.bindings.NotificationCallback
import com.oppzippy.openscq30.lib.bindings.OpenScq30Device
import com.oppzippy.openscq30.lib.wrapper.ConnectionStatus
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.update

class DeviceConnectionManager(val device: OpenScq30Device) : AutoCloseable {
    val connectionStatusFlow = MutableStateFlow(ConnectionStatus.Connected)

    val watchForChangeNotification = MutableStateFlow(0)

    private val mutableEvents = MutableSharedFlow<String>(
        extraBufferCapacity = 16,
        onBufferOverflow = BufferOverflow.DROP_OLDEST,
    )
    val events = mutableEvents.asSharedFlow()

    init {
        device.setDeviceEventCallback(object : DeviceEventCallback {
            override fun onEvent(event: String) {
                mutableEvents.tryEmit(event)
            }
        })
        device.setConnectionStatusCallback(
            object : ConnectionStatusCallback {
                override fun onChange(connectionStatus: ConnectionStatus) {
                    connectionStatusFlow.value = connectionStatus
                }
            },
        )
        device.setWatchForChangesCallback(
            object : NotificationCallback {
                override fun onNotify() {
                    watchForChangeNotification.update { it + 1 }
                }
            },
        )
    }

    override fun close() {
        device.close()
    }
}

package com.oppzippy.openscq30.features.customactions

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class ActionGateTest {
    private val action = CustomActionConfig(true, "paseo://live-voice", "sh.paseo.assembly")

    @Test
    fun validatesLinksWithoutAcceptingIntentPayloads() {
        assertTrue(action.isValid())
        assertTrue(CustomActionConfig(uri = "https://example.com").isValid())
        for (uri in listOf(
            "",
            "no-scheme",
            "intent://host#Intent;end",
            "file:///private",
            "content://private",
            "javascript:alert(1)",
            "data:text/plain,test",
        )) {
            assertFalse(CustomActionConfig(uri = uri).isValid())
        }
        assertFalse(action.copy(packageName = "bad package").isValid())
    }

    @Test
    fun ignoresDisabledAndUnrelatedEvents() {
        val gate = ActionGate { 0 }
        assertFalse(gate.accept(ASSISTANT_EVENT, action.copy(enabled = false)))
        assertFalse(gate.accept("state-change", action))
        assertFalse(gate.accept(ASSISTANT_EVENT, action.copy(uri = "invalid")))
        assertTrue(gate.accept(ASSISTANT_EVENT, action))
    }

    @Test
    fun suppressesDuplicatePressesButAllowsTheNextGesture() {
        var now = 0L
        val gate = ActionGate { now }
        assertTrue(gate.accept(ASSISTANT_EVENT, action))
        now = 999
        assertFalse(gate.accept(ASSISTANT_EVENT, action))
        now = 1000
        assertTrue(gate.accept(ASSISTANT_EVENT, action))
    }

    @Test
    fun onlyAcceptsTheSelectedTrigger() {
        val gate = ActionGate { 0 }
        val soundMode = action.copy(trigger = CustomActionTrigger.SOUND_MODE)
        assertFalse(gate.accept(SOUND_MODE_EVENT, action))
        assertFalse(gate.accept(ASSISTANT_EVENT, soundMode))
        assertTrue(gate.accept(SOUND_MODE_EVENT, soundMode))
        assertFalse(gate.accept(SOUND_MODE_EVENT, soundMode))
    }
}

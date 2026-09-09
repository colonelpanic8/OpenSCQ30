package com.oppzippy.openscq30.ui.settings

import android.content.Intent
import android.provider.Settings
import android.widget.Toast
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.selection.selectable
import androidx.compose.material3.Button
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.Role
import androidx.core.net.toUri
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.ViewModel
import com.oppzippy.openscq30.R
import com.oppzippy.openscq30.features.customactions.CustomActionConfig
import com.oppzippy.openscq30.features.customactions.CustomActionLauncher
import com.oppzippy.openscq30.features.customactions.CustomActionStore
import com.oppzippy.openscq30.features.customactions.CustomActionTrigger
import com.oppzippy.openscq30.ui.utils.LabeledSwitch
import dagger.hilt.android.lifecycle.HiltViewModel
import javax.inject.Inject

@HiltViewModel
class CustomActionViewModel @Inject constructor(
    private val store: CustomActionStore,
    val launcher: CustomActionLauncher,
) : ViewModel() {
    val config = store.config
    fun save(config: CustomActionConfig) = store.save(config)
}

@Composable
fun CustomActionSettings(viewModel: CustomActionViewModel = hiltViewModel()) {
    val saved by viewModel.config.collectAsState()
    var uri by remember(saved.uri) { mutableStateOf(saved.uri) }
    var packageName by remember(saved.packageName) { mutableStateOf(saved.packageName) }
    var trigger by remember(saved.trigger) { mutableStateOf(saved.trigger) }
    val context = LocalContext.current
    val config = CustomActionConfig(saved.enabled, uri.trim(), packageName.trim(), trigger)
    Column {
        Text(stringResource(R.string.custom_action_title))
        Text(stringResource(R.string.custom_action_description))
        Text(stringResource(R.string.custom_action_trigger))
        CustomActionTrigger.entries.forEach { option ->
            Row(
                modifier = Modifier.selectable(
                    selected = trigger == option,
                    role = Role.RadioButton,
                    onClick = { trigger = option },
                ),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                RadioButton(selected = trigger == option, onClick = null)
                Text(
                    stringResource(
                        if (option == CustomActionTrigger.ASSISTANT) {
                            R.string.custom_action_trigger_assistant
                        } else {
                            R.string.custom_action_trigger_sound_mode
                        },
                    ),
                )
            }
        }
        if (trigger == CustomActionTrigger.SOUND_MODE) {
            Text(stringResource(R.string.custom_action_sound_mode_description))
        }
        OutlinedTextField(
            value = uri,
            onValueChange = { uri = it },
            label = { Text(stringResource(R.string.custom_action_uri)) },
            singleLine = true,
        )
        OutlinedTextField(
            value = packageName,
            onValueChange = { packageName = it },
            label = { Text(stringResource(R.string.custom_action_package)) },
            singleLine = true,
        )
        Button(onClick = { viewModel.save(config) }, enabled = config.isValid()) {
            Text(stringResource(R.string.custom_action_save))
        }
        LabeledSwitch(
            label = stringResource(R.string.custom_action_enable),
            isChecked = saved.enabled,
            onCheckedChange = { enabled ->
                if (!enabled) {
                    viewModel.save(saved.copy(enabled = false))
                } else if (config.isValid()) {
                    viewModel.save(config.copy(enabled = true))
                } else {
                    Toast.makeText(context, R.string.custom_action_invalid, Toast.LENGTH_SHORT).show()
                }
            },
        )
        Button(onClick = {
            context.startActivity(
                Intent(Settings.ACTION_MANAGE_OVERLAY_PERMISSION, "package:${context.packageName}".toUri()),
            )
        }) {
            Text(stringResource(R.string.custom_action_permission))
        }
        Text(stringResource(R.string.custom_action_permission_description))
        Button(onClick = {
            if (!viewModel.launcher.launch(config)) {
                Toast.makeText(context, R.string.custom_action_failed, Toast.LENGTH_SHORT).show()
            }
        }, enabled = config.isValid()) {
            Text(stringResource(R.string.custom_action_test))
        }
    }
}

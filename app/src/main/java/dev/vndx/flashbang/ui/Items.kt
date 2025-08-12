package dev.vndx.flashbang.ui

import androidx.compose.animation.ExperimentalSharedTransitionApi
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedCard
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Surface
import androidx.compose.material3.SwipeToDismissBox
import androidx.compose.material3.SwipeToDismissBoxValue
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TextField
import androidx.compose.material3.rememberSwipeToDismissBoxState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.pluralStringResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.lifecycle.viewmodel.compose.viewModel
import dev.vndx.flashbang.R
import dev.vndx.flashbang.data.dateTimeFormatter
import java.time.LocalDate

// Taken from https://proandroiddev.com/seamless-shimmer-integration-with-existing-compose-code-b95cc3bbcd17

@Composable
private fun IconDisplay(painter: Painter, contentDescription: String? = null) {
    Icon(
        modifier = Modifier.Companion
            .background(
                MaterialTheme.colorScheme.primaryContainer,
                shape = RoundedCornerShape(Sizes.cornerRadiusMedium)
            )
            .padding(Sizes.spacingMedium),
        painter = painter,
        contentDescription = contentDescription,
        tint = MaterialTheme.colorScheme.onBackground,
    )
}

@Composable
fun Directory(
    name: String,
    cards: Int,
    modifier: Modifier = Modifier,
    onClick: () -> Unit = {},
    content: @Composable RowScope.() -> Unit = {},
) {
    val context = LocalContext.current
    Surface(
        color = MaterialTheme.colorScheme.background,
        modifier = modifier
            .clickable(onClick = onClick)
    ) {
        Row(
            verticalAlignment = Alignment.Companion.Top,
            modifier = Modifier.Companion
                .fillMaxWidth()
                .padding(Sizes.spacingMedium)
        ) {
            IconDisplay(painterResource(R.drawable.outline_folder_32))
            Box(modifier = Modifier.Companion.weight(1f)) {
                Column(
                    modifier = Modifier.Companion
                        .padding(Sizes.spacingLarge, Sizes.spacingTiny)
                        .fillMaxWidth(),
                    verticalArrangement = Arrangement.spacedBy(
                        Sizes.spacingTiny,
                        alignment = Alignment.Companion.Top
                    )
                ) {
                    Text(name, style = MaterialTheme.typography.headlineSmall)
                    Text(
                        pluralStringResource(R.plurals.card_count, cards, cards),
                        style = MaterialTheme.typography.bodyLarge,
                        color = MaterialTheme.colorScheme.secondary
                    )
                }
            }

            content()
        }
    }
}

@Composable
fun Flashcard(
    name: String,
    scheduled: Boolean,
    modifier: Modifier = Modifier,
    onLongClick: () -> Unit = {},
    onClick: () -> Unit = {},
    content: @Composable RowScope.() -> Unit = {},
) {
    val context = LocalContext.current
    Surface(
        color = MaterialTheme.colorScheme.background,
        modifier = modifier
            .combinedClickable(onClick = onClick, onLongClick = onLongClick)
    ) {
        Row(
            verticalAlignment = Alignment.Companion.Top,
            modifier = Modifier.Companion
                .fillMaxWidth()
                .padding(Sizes.spacingMedium)
        ) {
            Icon(
                modifier = Modifier.Companion
                    .shimmerable()
                    .background(
                        MaterialTheme.colorScheme.primaryContainer,
                        shape = androidx.compose.foundation.shape.RoundedCornerShape(Sizes.cornerRadiusMedium)
                    )
                    .padding(Sizes.spacingMedium)
                    .offset(0.dp, 2.dp),
                painter = painterResource(R.drawable.outline_edit_note_32),
                contentDescription = null,
                tint = MaterialTheme.colorScheme.onBackground,
            )
            Box(
                modifier = Modifier.Companion
                    .weight(1f)
                    .align(Alignment.Companion.CenterVertically)
            ) {
                Column(
                    modifier = Modifier.Companion
                        .padding(Sizes.spacingLarge, Sizes.spacingTiny)
                        .fillMaxWidth(),
                    verticalArrangement = Arrangement.spacedBy(
                        Sizes.spacingTiny,
                        alignment = Alignment.Companion.CenterVertically
                    )
                ) {
                    Text(
                        name,
                        modifier = Modifier.shimmerable(),
                        style = MaterialTheme.typography.headlineSmall
                    )
                    if (scheduled) {
                        Text(
                            stringResource(R.string.scheduled),
                            modifier = Modifier.shimmerable(),
                            style = MaterialTheme.typography.bodyLarge,
                            color = MaterialTheme.colorScheme.secondary
                        )
                    }
                }
            }

            content()
        }
    }
}

@OptIn(ExperimentalSharedTransitionApi::class)
@Composable
fun Study(
    name: String,
    cards: Int,
    handle: Int,
    description: String,
    date: LocalDate,
    progress: Float,
    onEdit: () -> Unit = {},
    onResume: () -> Unit = {},
    onSwipe: () -> Unit = {},
) {
    val dismissState = rememberSwipeToDismissBoxState(
        positionalThreshold = { totalDistance -> totalDistance * 0.6f },
        confirmValueChange = {
            if (it == SwipeToDismissBoxValue.StartToEnd) {
                onSwipe()
            }

            it == SwipeToDismissBoxValue.StartToEnd
        }
    )
    val preferencesState by viewModel<SettingsViewModel>().preferences.collectAsState()
    val preferences = preferencesState.preferences
    val relative = true

    SwipeToDismissBox(
        state = dismissState,
        enableDismissFromEndToStart = false,
        backgroundContent = {}
    ) {
        OutlinedCard(
            modifier = Modifier
                .fillMaxWidth()
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(Sizes.spacingMedium),
                verticalArrangement = Arrangement.spacedBy(Sizes.spacingMedium)
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(Sizes.spacingMedium)
                ) {
                    IconDisplay(painterResource(R.drawable.outline_note_stack_32))
                    TitleSubtitleStack(
                        name,
                        pluralStringResource(R.plurals.card_count, cards, cards)
                    )
                    CircularProgressIndicator(
                        progress = { progress },
                        modifier = Modifier.padding(0.dp, 0.dp, Sizes.spacingMedium)
                    )
                }

                Text(description)

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.Bottom,
                    horizontalArrangement = Arrangement.spacedBy(Sizes.spacingMedium)
                ) {
                    Box(modifier = Modifier.weight(1f)) {
                        Text(
                            formatRelativeDate(
                                date,
                                relative,
                                preferences.dateFormat.dateTimeFormatter()
                            ),
                            style = MaterialTheme.typography.bodySmall
                        )
                    }
                    Button(onClick = onResume) {
                        Text(stringResource(R.string.resume))
                    }
                    Button(onClick = onEdit) {
                        Text(stringResource(R.string.edit))
                    }
                }
            }
        }
    }
}

@Composable
private fun RowScope.TitleSubtitleStack(
    title: String,
    subtitle: String? = null
) {
    Box(modifier = Modifier.weight(1f)) {
        Column(
            verticalArrangement = Arrangement.spacedBy(Sizes.spacingMedium)
        ) {
            Text(
                title,
                style = MaterialTheme.typography.titleMedium,
            )
            subtitle?.let {
                Text(
                    it,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.secondary
                )
            }
        }
    }
}

@Composable
fun SettingsCategory(
    painter: Painter,
    title: String,
    subtitle: String? = null,
    contentDescription: String? = null,
    onClick: () -> Unit = {}
) {
    Surface(
        onClick = onClick,
    ) {
        Row(
            modifier = Modifier
                .padding(Sizes.spacingMedium)
                .fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(Sizes.spacingMedium)
        ) {
            IconDisplay(painter)
            TitleSubtitleStack(title, subtitle)
        }
    }
}

@Composable
fun SettingsSwitch(
    title: String,
    checked: Boolean,
    subtitle: String? = null,
    onCheckedChange: (Boolean) -> Unit = {}
) {
    Surface(
        onClick = { onCheckedChange(!checked) },
    ) {
        Row(
            modifier = Modifier
                .padding(Sizes.spacingMedium)
                .fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(Sizes.spacingMedium)
        ) {
            TitleSubtitleStack(title, subtitle)
            Switch(
                checked = checked,
                onCheckedChange = onCheckedChange
            )
        }
    }
}

@Composable
private fun SettingDialogItem(
    dialogOpen: Boolean,
    onOpenDialog: () -> Unit = {},
    onDismissDialog: () -> Unit = {},
    dialogContent: @Composable () -> Unit = {},
    content: @Composable () -> Unit = {}
) {
    Surface(
        onClick = onOpenDialog,
    ) {
        content()
    }
    if (dialogOpen) {
        Dialog(
            onDismissRequest = onDismissDialog
        ) {
            Card(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(Sizes.spacingLarge),
                shape = RoundedCornerShape(Sizes.cornerRadiusHuge)
            ) {
                dialogContent()
            }
        }
    }
}

@Composable
fun <T> SettingsSelect(
    title: String,
    selected: String,
    options: List<Pair<String, T>>,
    subtitle: String? = null,
    onSelect: (Pair<String, T>) -> Unit,
) {
    var dialogOpen by remember { mutableStateOf(false) }

    SettingDialogItem(
        dialogOpen = dialogOpen,
        onOpenDialog = { dialogOpen = true },
        onDismissDialog = { dialogOpen = false },
        dialogContent = {
            LazyColumn(
                modifier = Modifier.padding(Sizes.spacingMedium),
            ) {
                item {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(Sizes.spacingMedium),
                        horizontalArrangement = Arrangement.Center
                    ) {
                        Text(title, style = MaterialTheme.typography.headlineMedium)
                    }
                }

                items(options) { (name, value) ->
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .selectable(
                                selected = (selected == name),
                                onClick = { onSelect(Pair(name, value)) },
                                role = Role.RadioButton
                            )
                            .padding(Sizes.spacingLarge),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(Sizes.spacingMedium)
                    ) {
                        Box(modifier = Modifier.weight(1f)) {
                            Text(name, style = MaterialTheme.typography.headlineSmall)
                        }
                        RadioButton(
                            selected = (selected == name),
                            onClick = null
                        )
                    }
                }

                item {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.End,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        TextButton(onClick = {
                            dialogOpen = false
                        }) {
                            Text(
                                stringResource(R.string.cancel),
                                style = MaterialTheme.typography.bodyLarge
                            )
                        }
                    }
                }
            }
        }
    ) {
        Row(
            modifier = Modifier
                .padding(Sizes.spacingMedium)
                .fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(Sizes.spacingMedium)
        ) {
            TitleSubtitleStack(title, subtitle)
        }
    }
}

@Composable
fun SettingsTextField(
    title: String,
    value: String,
    subtitle: String? = null,
    onValueChange: (String) -> Unit = {}
) {
    var dialogOpen by remember { mutableStateOf(false) }
    var text by remember(value) { mutableStateOf(value) }

    SettingDialogItem(
        dialogOpen = dialogOpen,
        onOpenDialog = { dialogOpen = true },
        onDismissDialog = { dialogOpen = false },
        dialogContent = {
            Column(
                modifier = Modifier.padding(Sizes.spacingMedium),
            ) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(Sizes.spacingMedium),
                    horizontalArrangement = Arrangement.Center
                ) {
                    Text(title, style = MaterialTheme.typography.headlineMedium)
                }

                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(Sizes.spacingLarge),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(Sizes.spacingMedium)
                ) {
                    TextField(
                        value = text,
                        onValueChange = {
                            text = it
                        }
                    )
                }

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.End,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    TextButton(onClick = {
                        dialogOpen = false
                    }) {
                        Text(
                            stringResource(R.string.cancel),
                            style = MaterialTheme.typography.bodyLarge
                        )
                    }
                    TextButton(
                        onClick = {
                            onValueChange(text)
                            dialogOpen = false
                        },
                        enabled = (text != value)
                    ) {
                        Text(
                            stringResource(android.R.string.ok),
                            style = MaterialTheme.typography.bodyLarge
                        )
                    }
                }
            }
        }
    ) {
        Row(
            modifier = Modifier
                .padding(Sizes.spacingMedium)
                .fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(Sizes.spacingMedium)
        ) {
            TitleSubtitleStack(title, subtitle)
        }
    }
}

@Composable
fun SettingsAction(title: String, subtitle: String? = null, onClick: () -> Unit = {}) {
    Surface(
        onClick = onClick,
    ) {
        Row(
            modifier = Modifier
                .padding(Sizes.spacingMedium)
                .fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(Sizes.spacingMedium)
        ) {
            TitleSubtitleStack(title, subtitle)
        }
    }
}
package dev.vndx.flashbang.ui.screens

import androidx.compose.animation.ExperimentalSharedTransitionApi
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import dev.vndx.flashbang.R
import dev.vndx.flashbang.data.dateTimeFormatter
import dev.vndx.flashbang.domain.Card
import dev.vndx.flashbang.domain.Tag
import dev.vndx.flashbang.ui.CardsUiState
import dev.vndx.flashbang.ui.CardsViewModel
import dev.vndx.flashbang.ui.Directory
import dev.vndx.flashbang.ui.Flashcard
import dev.vndx.flashbang.ui.SettingsViewModel
import dev.vndx.flashbang.ui.Sizes
import dev.vndx.flashbang.ui.StudiesState
import dev.vndx.flashbang.ui.StudiesViewModel
import dev.vndx.flashbang.ui.formatRelativeDate
import kotlinx.serialization.Serializable
import java.time.LocalDate

@Serializable
class EditStudyScreen(val id: Long) : Screen {
    override fun tab() = Tab.Study

    @OptIn(ExperimentalSharedTransitionApi::class)
    @Composable
    override fun Compose(onNavigate: (Screen) -> Unit, onBack: (Int?) -> Unit) {
        val viewModel: StudiesViewModel = viewModel()
        val cardsViewModel: CardsViewModel = viewModel()
        val settingsViewModel: SettingsViewModel = viewModel()

        val state by viewModel.studiesState.collectAsState()
        val cardsState by cardsViewModel.uiState.collectAsState()
        val preferencesState by settingsViewModel.preferences.collectAsState()

        if (state !is StudiesState.Success) {
            Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                CircularProgressIndicator()
            }
            return
        }

        val study = (state as StudiesState.Success).studies[id]
        if (study == null) {
            onBack(null)
            return
        }

        var name by remember(study.name) { mutableStateOf(study.name) }
        var showDeleteDialog by remember { mutableStateOf(false) }

        val currentStudy by rememberUpdatedState(study)
        val currentName by rememberUpdatedState(name)

        fun saveName() {
            if (currentName != currentStudy.name) {
                viewModel.renameStudy(currentStudy, currentName)
            }
        }

        DisposableEffect(Unit) {
            onDispose {
                saveName()
            }
        }

        val summary by remember(study, cardsState) {
            derivedStateOf {
                study.getOrBuildSelectionSummary(cardsState)
            }
        }

        val progress = if (study.selection.isNotEmpty()) {
            study.reviews.size.toFloat() / study.selection.size.toFloat()
        } else {
            0f
        }

        Column(
            modifier = Modifier
                .padding(Sizes.spacingMedium)
                .fillMaxSize(),
            verticalArrangement = Arrangement.spacedBy(Sizes.spacingMedium)
        ) {
            OutlinedTextField(
                modifier = Modifier
                    .fillMaxWidth()
                    .onFocusChanged { focusState ->
                        if (!focusState.isFocused) {
                            saveName()
                        }
                    },
                value = name,
                onValueChange = { name = it },
                label = { Text(stringResource(R.string.study_name_label)) },
                singleLine = true
            )

            Text(
                text = formatRelativeDate(
                    study.timestamp.toLocalDate(),
                    true,
                    preferencesState.preferences.dateFormat.dateTimeFormatter()
                ),
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )

            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(vertical = Sizes.spacingLarge),
                contentAlignment = Alignment.Center
            ) {
                CircularProgressIndicator(
                    progress = { progress },
                    modifier = Modifier.size(120.dp),
                    strokeWidth = 8.dp,
                )
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Text(
                        text = "${(progress * 100).toInt()}%",
                        style = MaterialTheme.typography.headlineMedium
                    )
                    Text(
                        text = "${study.reviews.size} / ${study.selection.size}",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(Sizes.spacingMedium)
            ) {
                Button(
                    modifier = Modifier.weight(1f),
                    shape = RoundedCornerShape(Sizes.cornerRadiusLarge),
                    onClick = {
                        saveName()
                        onNavigate(ReviewScreen(study))
                    }
                ) {
                    Text(stringResource(R.string.resume))
                }

                OutlinedButton(
                    modifier = Modifier.weight(1f),
                    shape = RoundedCornerShape(Sizes.cornerRadiusLarge),
                    onClick = { showDeleteDialog = true }
                ) {
                    Text("Delete")
                }
            }

            Text(
                text = stringResource(R.string.study_selection_label),
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(top = Sizes.spacingMedium)
            )

            LazyColumn(
                verticalArrangement = Arrangement.spacedBy(Sizes.spacingSmall),
                modifier = Modifier.weight(1f)
            ) {
                if (summary.isEmpty()) {
                    item {
                        Text(
                            text = stringResource(R.string.no_selected_cards),
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(Sizes.spacingLarge),
                            textAlign = TextAlign.Center
                        )
                    }
                } else {
                    items(summary, key = { item ->
                        when (item) {
                            is Card -> item.id
                            is Tag -> item.fullPath
                            else -> item.itemName
                        }
                    }) { item ->
                        when (item) {
                            is Card -> {
                                Flashcard(
                                    name = item.name,
                                    scheduled = item.scheduledFor == LocalDate.now(),
                                )
                            }

                            is Tag -> {
                                Directory(
                                    name = item.name,
                                    cards = item.indirectCards.size,
                                )
                            }
                        }
                    }
                }
            }
        }

        if (showDeleteDialog) {
            AlertDialog(
                onDismissRequest = { showDeleteDialog = false },
                title = { Text(stringResource(R.string.study_deletion_title)) },
                text = { Text(stringResource(R.string.study_deletion_content)) },
                confirmButton = {
                    TextButton(
                        onClick = {
                            viewModel.deleteStudy(study)
                            showDeleteDialog = false
                            onBack(1)
                        }
                    ) {
                        Text(stringResource(R.string.confirm))
                    }
                },
                dismissButton = {
                    TextButton(onClick = { showDeleteDialog = false }) {
                        Text(stringResource(R.string.cancel))
                    }
                }
            )
        }
    }
}

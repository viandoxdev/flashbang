package dev.vndx.flashbang.ui.screens

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import dev.vndx.flashbang.R
import dev.vndx.flashbang.data.dateTimeFormatter
import dev.vndx.flashbang.domain.Card
import dev.vndx.flashbang.domain.Study
import dev.vndx.flashbang.domain.Tag
import dev.vndx.flashbang.ui.Directory
import dev.vndx.flashbang.ui.Flashcard
import dev.vndx.flashbang.ui.SettingsViewModel
import dev.vndx.flashbang.ui.Sizes
import dev.vndx.flashbang.ui.StudiesViewModel
import dev.vndx.flashbang.ui.formatRelativeDate
import kotlinx.serialization.Serializable
import java.time.LocalDate

@Serializable
class CreateStudyScreen : Screen {
    override fun tab(): Tab = Tab.Study

    @Composable
    override fun Compose(onNavigate: (Screen) -> Unit, onBack: (Int?) -> Unit) {
        val preferencesState by viewModel<SettingsViewModel>().preferences.collectAsState()
        val preferences = preferencesState.preferences
        val studiesViewModel = viewModel<StudiesViewModel>()
        val selectionViewModel = viewModel<SelectionViewModel>()
        val count = selectionViewModel.selection.size
        val summary by remember(selectionViewModel) {
            derivedStateOf {
                Study.buildSelectionSummary(selectionViewModel.selection)
            }
        }
        var name by remember { mutableStateOf("") }
        val placeholderName = stringResource(
            R.string.study_default_name, formatRelativeDate(
                LocalDate.now(), false, preferences.dateFormat.dateTimeFormatter()
            )
        )

        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(Sizes.spacingMedium),
            verticalArrangement = Arrangement.spacedBy(Sizes.spacingMedium),
        ) {
            Text(
                stringResource(R.string.study_name_label),
                style = MaterialTheme.typography.labelMedium
            )

            OutlinedTextField(
                modifier = Modifier
                    .clearFocusOnKeyboardDismiss()
                    .fillMaxWidth(),
                shape = RoundedCornerShape(Sizes.cornerRadiusLarge),
                value = name,
                onValueChange = { name = it },
                placeholder = {
                    Text(
                        placeholderName
                    )
                })

            Row(
                modifier = Modifier
                    .padding(0.dp, Sizes.spacingLarge, 0.dp)
                    .fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    stringResource(R.string.study_selection_label),
                    style = MaterialTheme.typography.labelLarge
                )

                TextButton(onClick = {
                    onNavigate(SelectionScreen(selectionViewModel))
                }) {
                    Text(stringResource(R.string.edit), style = MaterialTheme.typography.labelMedium)
                }
            }

            Box(modifier = Modifier.weight(1f)) {
                if (selectionViewModel.isEmpty()) {
                    Text(
                        modifier = Modifier.align(Alignment.TopCenter)
                            .padding(Sizes.spacingLarge),
                        textAlign = TextAlign.Center,
                        text = stringResource(R.string.no_selected_cards),
                        style = MaterialTheme.typography.bodyLarge,
                        color = MaterialTheme.colorScheme.outline
                    )
                } else {
                    LazyColumn {
                        items(summary) { item ->
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

                                else -> {
                                    Text("what")
                                }
                            }
                        }
                    }
                }
            }

            Button(
                modifier = Modifier.fillMaxWidth(),
                shape = RoundedCornerShape(Sizes.cornerRadiusLarge),
                enabled = !selectionViewModel.isEmpty(),
                onClick = {
                    val selection = selectionViewModel.selection.toList().map { it.id }
                    selectionViewModel.clear()
                    val studyName = name.ifEmpty {
                        placeholderName
                    }

                    name = ""

                    val study = studiesViewModel.createStudy(selection, studyName)

                    onNavigate(ReviewScreen(study))
                },
            ) {
                Text(
                    modifier = Modifier.padding(Sizes.spacingSmall),
                    text = stringResource(R.string.study_create)
                )
            }
        }

    }
}
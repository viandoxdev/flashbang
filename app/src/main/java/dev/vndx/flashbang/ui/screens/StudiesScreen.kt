package dev.vndx.flashbang.ui.screens

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import dev.vndx.flashbang.R
import dev.vndx.flashbang.ui.CardsUiState
import dev.vndx.flashbang.ui.CardsViewModel
import dev.vndx.flashbang.ui.ShimmerProvider
import dev.vndx.flashbang.ui.Sizes
import dev.vndx.flashbang.ui.StudiesViewModel
import dev.vndx.flashbang.ui.Study
import kotlinx.serialization.Serializable
import java.time.LocalDate

data class FakeStudy(val id: Long, val name: String, val description: String)

val FakeStudies = listOf(
    FakeStudy(
        id = 0,
        name = "Super uber study",
        description = "This is a study, it contains a selection of cards, although this one doesn't exactly as it is meant as a placeholder for loading screen, this text won't even be readable as it'll be replaced with a shimmer effect for styling purposes."
    ), FakeStudy(
        id = 1,
        name = "The uhh other one",
        description = "I'm unfortunately all out of ideas, I mean I could just write gibberish, all that matters is that the text be of a different length after all, but I'm not too fond of that idea as it would look bad in code, So I'm just writing stuff without much meaning to make it look like text."
    )
)

@Serializable
class StudiesScreen() : Screen {
    override fun tab(): Tab = Tab.Study
    override fun showTabs(): Boolean = true
    override fun isHomeScreen(): Boolean = true

    @Composable
    override fun Compose(onNavigate: (Screen) -> Unit, onBack: () -> Unit) {
        val cardsState by viewModel<CardsViewModel>().uiState.collectAsState()
        val viewModel: StudiesViewModel = viewModel()
        val state by viewModel.studiesState.collectAsState()

        val studies = state.studies.values.toList()

        if (cardsState is CardsUiState.Loading) {
            ShimmerProvider {
                LazyColumn(
                    modifier = Modifier.Companion
                        .fillMaxSize()
                        .padding(Sizes.spacingMedium, 0.dp),
                    verticalArrangement = Arrangement.spacedBy(Sizes.spacingMedium)
                ) {
                    items(FakeStudies, { it.id }) { study ->
                        Study(
                            name = study.name,
                            cards = 0,
                            id = study.id,
                            description = study.description,
                            date = LocalDate.MIN,
                            progress = 0.5f,
                            onEdit = {})
                    }
                }
            }
        } else {
            // Only display this when cardState is loaded because we need it to access the cards' name
            // when building the selection summary
            LazyColumn(
                modifier = Modifier.Companion
                    .fillMaxSize()
                    .padding(Sizes.spacingMedium, 0.dp),
                verticalArrangement = Arrangement.spacedBy(Sizes.spacingMedium)
            ) {
                items(studies, { it.id }) { study ->
                    Study(
                        name = study.name,
                        cards = study.selection.size,
                        id = study.id,
                        description = stringResource(
                            R.string.selected,
                            study.getOrBuildSelectionSummary(cardsState)
                                .joinToString(", ") { it.itemName }),
                        date = study.timestamp.toLocalDate(),
                        progress = study.reviews.size.toFloat() / study.selection.size.toFloat(),
                        onEdit = { onNavigate(EditStudyScreen(study.id)) },
                        onDelete = {
                            viewModel.deleteStudy(study)
                        }
                    )
                }
            }
        }

        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(Sizes.spacingLarge)
        ) {
            FloatingActionButton(
                modifier = Modifier.align(Alignment.BottomEnd),
                onClick = { onNavigate(CreateStudyScreen()) }) {
                Icon(
                    modifier = Modifier.padding(Sizes.spacingLarge),
                    painter = painterResource(R.drawable.outline_add_32),
                    contentDescription = null
                )
            }
        }
    }
}
package dev.vndx.flashbang.ui.screens

import dev.vndx.flashbang.R
import androidx.activity.compose.LocalActivity
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.lifecycle.ViewModelStoreOwner
import androidx.lifecycle.viewmodel.compose.viewModel
import dev.vndx.flashbang.Rating
import dev.vndx.flashbang.ui.CardsViewModel
import dev.vndx.flashbang.ui.Sizes
import dev.vndx.flashbang.ui.StudiesState
import dev.vndx.flashbang.ui.StudiesViewModel
import kotlinx.serialization.Serializable
import java.time.LocalDateTime
import java.time.ZoneOffset
import java.time.format.DateTimeFormatter
import java.time.format.FormatStyle

@Serializable
data class CardHistoryScreen(val cardId: String) : Screen {
    override fun tab(): Tab = Tab.Cards
    override fun showTabs(): Boolean = false

    @Composable
    override fun ComposeTopBarAction(onNavigate: (Screen) -> Unit, onBack: (Int?) -> Unit) {
    }

    @Composable
    override fun Compose(onNavigate: (Screen) -> Unit, onBack: (Int?) -> Unit) {
        val studiesViewModel = viewModel<StudiesViewModel>(viewModelStoreOwner = LocalActivity.current as ViewModelStoreOwner)
        val cardsViewModel = viewModel<CardsViewModel>(viewModelStoreOwner = LocalActivity.current as ViewModelStoreOwner)

        val cardsState by cardsViewModel.uiState.collectAsState()
        val studiesState by studiesViewModel.studiesState.collectAsState()

        val card = cardsState.cards[cardId]

        if (card == null) {
            Column(
                modifier = Modifier.fillMaxSize(),
                verticalArrangement = Arrangement.Center,
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                Text("Card not found", style = MaterialTheme.typography.headlineMedium)
            }
            return
        }

        val memory = (studiesState as? StudiesState.Success)?.proto?.memoryMap?.get(cardId)
        val reviews = memory?.reviewsList?.sortedByDescending { it.timestamp } ?: emptyList()

        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(Sizes.spacingMedium),
            verticalArrangement = Arrangement.spacedBy(Sizes.spacingMedium)
        ) {
            Text(
                text = card.name,
                style = MaterialTheme.typography.headlineMedium
            )

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween
            ) {
                Column {
                    Text("Stability", style = MaterialTheme.typography.labelLarge)
                    Text(String.format("%.2f", memory?.stability ?: 0f), style = MaterialTheme.typography.bodyLarge)
                }
                Column {
                    Text("Difficulty", style = MaterialTheme.typography.labelLarge)
                    Text(String.format("%.2f", memory?.difficulty ?: 0f), style = MaterialTheme.typography.bodyLarge)
                }
            }

            Button(
                onClick = { studiesViewModel.clearCardHistory(card, cardId) },
                enabled = reviews.isNotEmpty(),
                modifier = Modifier.fillMaxWidth()
            ) {
                Text("Clear History")
            }

            Text("Reviews (${reviews.size})", style = MaterialTheme.typography.titleMedium)

            LazyColumn(
                modifier = Modifier.fillMaxSize(),
                verticalArrangement = Arrangement.spacedBy(Sizes.spacingSmall)
            ) {
                items(reviews, key = { it.timestamp }) { review ->
                    val date = LocalDateTime.ofEpochSecond(review.timestamp, 0, ZoneOffset.UTC)
                    val formatter = DateTimeFormatter.ofLocalizedDateTime(FormatStyle.MEDIUM, FormatStyle.SHORT)

                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Column(modifier = Modifier.weight(1f)) {
                            Text(date.format(formatter), style = MaterialTheme.typography.bodyMedium)
                            Text(
                                text = when (review.rating) {
                                    Rating.RATING_AGAIN -> "Again"
                                    Rating.RATING_HARD -> "Hard"
                                    Rating.RATING_GOOD -> "Good"
                                    Rating.RATING_EASY -> "Easy"
                                    else -> "Unknown"
                                },
                                style = MaterialTheme.typography.bodySmall,
                                color = when (review.rating) {
                                    Rating.RATING_AGAIN -> MaterialTheme.colorScheme.error
                                    Rating.RATING_HARD -> MaterialTheme.colorScheme.tertiary
                                    Rating.RATING_GOOD -> MaterialTheme.colorScheme.primary
                                    Rating.RATING_EASY -> MaterialTheme.colorScheme.secondary
                                    else -> MaterialTheme.colorScheme.onSurface
                                }
                            )
                        }
                        IconButton(onClick = { studiesViewModel.deleteCardReview(card, cardId, review.timestamp) }) {
                            Icon(
                                painter = painterResource(R.drawable.outline_delete_32),
                                contentDescription = "Delete review",
                                tint = MaterialTheme.colorScheme.error
                            )
                        }
                    }
                }
            }
        }
    }
}

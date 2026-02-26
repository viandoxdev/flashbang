package dev.vndx.flashbang.ui.screens

import android.util.Log
import androidx.annotation.StringRes
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.requiredWidth
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonColors
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.stringResource
import androidx.lifecycle.viewmodel.compose.viewModel
import coil3.compose.AsyncImage
import coil3.request.ImageRequest
import coil3.svg.SvgDecoder
import dev.vndx.flashbang.R
import dev.vndx.flashbang.Rating
import dev.vndx.flashbang.TAG
import dev.vndx.flashbang.domain.Study
import dev.vndx.flashbang.ui.CardsUiState
import dev.vndx.flashbang.ui.CardsViewModel
import dev.vndx.flashbang.ui.SettingsViewModel
import dev.vndx.flashbang.ui.Sizes
import dev.vndx.flashbang.ui.StudiesState
import androidx.compose.runtime.produceState
import dev.vndx.flashbang.ui.StudiesViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.Serializable
import uniffi.mobile.SourceConfig
import java.nio.ByteBuffer
import kotlin.math.roundToInt

@Composable
fun RowScope.ReviewButton(@StringRes string: Int, color: Color, onClick: () -> Unit) {
    Button(
        colors = ButtonDefaults.buttonColors(containerColor = color),
        modifier = Modifier
            .weight(1f)
            .fillMaxHeight(),
        shape = RoundedCornerShape(Sizes.cornerRadiusMedium),
        contentPadding = PaddingValues(Sizes.spacingSmall),
        onClick = onClick
    ) {
        Text(stringResource(string), style = MaterialTheme.typography.labelMedium)
    }
}

@Serializable
class ReviewScreen(val study: Study) : Screen {
    init {
        Log.w(TAG, "New review screen for " + study.id.toString())
        Log.w(TAG, study.reviews.toString())
    }

    override fun tab(): Tab = Tab.Study

    // Pseudo random yet consistent sort of the cards
    val cards = study.selection.filter {
        if (!study.reviews.contains(it)) {
            return@filter true
        } else {
            Log.w(TAG, "Card $it has already been reviewed")
            return@filter false
        }
    }.sortedBy { (it.hashCode() + study.id).hashCode() }

    @Composable
    fun Loading() {
        Box(
            modifier = Modifier.fillMaxSize()
        ) {
            CircularProgressIndicator(
                modifier = Modifier.align(Alignment.Center)
            )
        }
    }

    @Composable
    override fun Compose(onNavigate: (Screen) -> Unit, onBack: (Int?) -> Unit) {
        val cardsViewModel = viewModel<CardsViewModel>()
        val studiesViewModel = viewModel<StudiesViewModel>()
        val cardsState by cardsViewModel.uiState.collectAsState()
        val studyState by studiesViewModel.studiesState.collectAsState()

        var page by remember { mutableIntStateOf(0) }

        if (cardsState is CardsUiState.Loading || studyState is StudiesState.Loading) {
            Loading()
            return
        }

        val card = cardsState.cards[cards[page / 2]]
        val onAnswer = page % 2 == 1

        val cardSources = cards.mapNotNull { cardsState.cards[it] }
        val pagesCount = cardSources.size * 2

        val ratingButton: @Composable RowScope.(Rating, Int, Color) -> Unit =
            { rating, resource, color ->
                ReviewButton(resource, color) {
                    Log.w(TAG, card?.id ?: "No card")
                    if (card != null) {
                        Log.w(TAG, "Reviewed card ${card.id}")
                        studiesViewModel.updateStudy(study, rating, card)
                    }
                    if (page >= pagesCount - 1) {
                        studiesViewModel.finalizeStudy(study)
                        onBack(1)
                        onNavigate(StudyFinishedScreen(study))
                    } else {
                        page += 1
                    }
                }
            }

        if (cardSources.size < cards.size) {
            Log.e(TAG, "Some cards could not be loaded.")
        }

        val density = LocalDensity.current
        val preferences by viewModel<SettingsViewModel>().preferences.collectAsState()

        Column(
            modifier = Modifier.fillMaxSize()
        ) {
            BoxWithConstraints(
                modifier = Modifier.weight(1f)
            ) {
                val color = MaterialTheme.colorScheme.onBackground
                val context = LocalContext.current
val pages = emptyList<ImageRequest>()

                if (pages == null) {
                    Loading()
                } else {
                    AsyncImage(
                        modifier = Modifier
                            .requiredWidth(this@BoxWithConstraints.maxWidth)
                            .verticalScroll(rememberScrollState())
                            .align(Alignment.Center)
                            .padding(bottom = Sizes.spacingMedium),
                        contentScale = ContentScale.FillWidth,
                        model = pages.getOrNull(page),
                        contentDescription = null,
                    )
                }
            }

            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier
                    .padding(Sizes.spacingMedium)
                    .height(Sizes.spacingHuge * 2),
                horizontalArrangement = Arrangement.spacedBy(Sizes.spacingSmall)
            ) {
                if (onAnswer) {
                    ratingButton(
                        Rating.RATING_AGAIN,
                        R.string.rating_again,
                        MaterialTheme.colorScheme.inverseSurface
                    )
                    ratingButton(
                        Rating.RATING_HARD, R.string.rating_hard, MaterialTheme.colorScheme.error
                    )
                    ratingButton(
                        Rating.RATING_GOOD, R.string.rating_good, MaterialTheme.colorScheme.tertiary
                    )
                    ratingButton(
                        Rating.RATING_EASY, R.string.rating_easy, MaterialTheme.colorScheme.primary
                    )
                } else {
                    ReviewButton(R.string.flip, Color.Unspecified) { page += 1 }
                }
            }
        }
    }
}

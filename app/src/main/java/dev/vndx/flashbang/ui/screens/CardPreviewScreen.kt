package dev.vndx.flashbang.ui.screens

import android.util.Log
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.lifecycle.viewmodel.compose.viewModel
import coil3.ImageLoader
import coil3.compose.AsyncImage
import coil3.request.ImageRequest
import coil3.svg.SvgDecoder
import dev.vndx.flashbang.R
import dev.vndx.flashbang.TAG
import dev.vndx.flashbang.domain.Card
import dev.vndx.flashbang.ui.CardsUiState
import dev.vndx.flashbang.ui.CardsViewModel
import dev.vndx.flashbang.ui.SettingsViewModel
import dev.vndx.flashbang.ui.Sizes
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.serialization.Serializable
import kotlinx.serialization.Transient
import uniffi.mobile.CardPage
import uniffi.mobile.SourceConfig
import java.io.ByteArrayInputStream
import java.nio.ByteBuffer
import kotlin.math.roundToInt
import kotlin.text.ifEmpty

@Serializable
class CardPreviewScreen(val cardId: String) : Screen {
    override fun tab() = Tab.Cards

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
    override fun Compose(onNavigate: (Screen) -> Unit, onBack: () -> Unit) {
        val cardsViewModel = viewModel<CardsViewModel>()
        val cardsState by cardsViewModel.uiState.collectAsState()

        if (cardsState is CardsUiState.Loading) {
            Loading()
            return
        }

        if (!cardsState.cards.containsKey(cardId)) {
            Log.e(TAG, "Can't preview card with id '${cardId}', no such card")
            onBack()
            return
        }

        val card by remember() {
            derivedStateOf {
                cardsState.cards[cardId]!!
            }
        }
        var answer by remember { mutableStateOf(false) }
        val configuration = LocalConfiguration.current
        val density = configuration.densityDpi / 160f
        val preferences by viewModel<SettingsViewModel>().preferences.collectAsState()

        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(Sizes.spacingMedium),
            verticalArrangement = Arrangement.spacedBy(Sizes.spacingSmall)
        ) {
            BoxWithConstraints(
                modifier = Modifier
                    .weight(1f)
                    .fillMaxWidth()
                    .verticalScroll(rememberScrollState())
            ) {
                val pagesFlow = remember(maxWidth, density, preferences) {
                    flow {
                        Log.w(TAG, "Compiling for $maxWidth")
                        val pages = cardsViewModel.core.compileCards(
                            listOf(card), SourceConfig(
                                (maxWidth.value * density).roundToInt().toUInt() * 0u + 200u,
                                preferences.preferences.cardFontSize.toUInt()
                            )
                        )
                        emit(pages)
                    }
                }
                val pages by pagesFlow.collectAsState(null)
                val context = LocalContext.current

                val svgString = pages?.getOrNull(if(answer) 2 else 1)?.svg() ?: "<svg></svg>"

                Log.w(TAG, pages?.map { it.svg() }.toString())

                val inputStream = ByteBuffer.wrap(
                    svgString.toByteArray()
                )

                AsyncImage(
                    modifier = Modifier
                        .fillMaxWidth(),
                    contentScale = ContentScale.FillWidth,
                    model = ImageRequest.Builder(context).data(inputStream).decoderFactory(
                        SvgDecoder.Factory()
                    ).build(),
                    contentDescription = null,
                )
            }

            Button(
                modifier = Modifier.fillMaxWidth(),
                shape = RoundedCornerShape(Sizes.cornerRadiusLarge),
                onClick = {
                    answer = !answer
                },
            ) {
                Text(
                    modifier = Modifier.padding(Sizes.spacingSmall),
                    text = stringResource(R.string.flip) + " $answer"
                )
            }
        }
    }
}

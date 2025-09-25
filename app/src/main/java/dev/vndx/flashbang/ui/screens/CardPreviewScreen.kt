package dev.vndx.flashbang.ui.screens

import android.util.Log
import androidx.compose.foundation.gestures.AnchoredDraggableState
import androidx.compose.foundation.gestures.DraggableAnchors
import androidx.compose.foundation.gestures.Orientation
import androidx.compose.foundation.gestures.anchoredDraggable
import androidx.compose.foundation.gestures.animateTo
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.requiredWidth
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.IntOffset
import androidx.lifecycle.viewmodel.compose.viewModel
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
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.launch
import kotlinx.serialization.Serializable
import kotlinx.serialization.Transient
import uniffi.mobile.SourceConfig
import java.nio.ByteBuffer
import kotlin.math.roundToInt

@Serializable
class CardPreviewScreen(val card: Card) : Screen {
    override fun tab() = Tab.Cards

    @Transient
    var draggableState: AnchoredDraggableState<Int>? = null
    var pagesCount: Int = 0

    @Composable
    override fun ComposeTopBarAction(onNavigate: (Screen) -> Unit, onBack: (Int?) -> Unit) {
        val scope = rememberCoroutineScope()
        IconButton(onClick = {
            if (pagesCount == 0) {
                return@IconButton
            }
            scope.launch {
                draggableState?.animateTo(((draggableState?.currentValue ?: -1) + 1) % pagesCount)
            }
        }) {
            Icon(
                painter = painterResource(R.drawable.outline_flip_32),
                contentDescription = null,
                tint = MaterialTheme.colorScheme.onBackground
            )
        }
    }

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
        val cardsState by cardsViewModel.uiState.collectAsState()

        if (cardsState is CardsUiState.Loading) {
            Loading()
            return
        }

        val density = LocalDensity.current
        val preferences by viewModel<SettingsViewModel>().preferences.collectAsState()

        BoxWithConstraints(
            modifier = Modifier
                .fillMaxSize()
                .padding(Sizes.spacingMedium),
        ) {
            val color = MaterialTheme.colorScheme.onBackground
            val pageWidthPixels = with(density) { maxWidth.toPx() }
            val context = LocalContext.current
            val pagesFlow = remember(maxWidth, density, preferences) {
                flow {
                    Log.w(TAG, "Compiling for $maxWidth")
                    val pages = cardsViewModel.core.compileCards(
                        listOf(card), SourceConfig(
                            maxWidth.value.roundToInt().toUInt(),
                            preferences.preferences.cardFontSize.toUInt(),
                            ((color.value shr 32) and 0xFFFFFFuL).toUInt()
                        )
                    ).filterIndexed { index, _ -> index > 0 }.map {
                        ImageRequest.Builder(context).data(ByteBuffer.wrap(it.svg().toByteArray()))
                            .decoderFactory(
                                SvgDecoder.Factory()
                            ).build()
                    }
                    emit(pages)
                }
            }
            val pages by pagesFlow.collectAsState(emptyList())

            draggableState = remember {
                AnchoredDraggableState(
                    initialValue = 0, anchors = DraggableAnchors {},
                )
            }
            val draggableState = draggableState!!
            pagesCount = pages.size

            draggableState.updateAnchors(DraggableAnchors {
                for (i in pages.indices) {
                    i at -pageWidthPixels * i
                }
            })

            Row(modifier = Modifier
                .fillMaxHeight()
                .requiredWidth(maxWidth * pagesCount)
                .offset {
                    IntOffset(
                        pageWidthPixels.roundToInt() * pagesCount / 4 + draggableState.offset.let { if (it.isNaN()) 0 else it.toInt() },
                        0
                    )
                }
                .anchoredDraggable(
                    orientation = Orientation.Horizontal, state = draggableState,
                ), verticalAlignment = Alignment.CenterVertically

            ) {
                for (page in pages) {
                    AsyncImage(
                        modifier = Modifier
                            .requiredWidth(this@BoxWithConstraints.maxWidth)
                            .verticalScroll(rememberScrollState())
                            .padding(bottom = Sizes.spacingHuge * 2),
                        contentScale = ContentScale.FillWidth,
                        model = page,
                        contentDescription = null,
                    )
                }
            }
        }
    }
}

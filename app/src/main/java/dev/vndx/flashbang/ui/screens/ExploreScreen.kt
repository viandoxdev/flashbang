package dev.vndx.flashbang.ui.screens

import androidx.activity.compose.LocalActivity
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.ime
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.LocalTextStyle
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SearchBar
import androidx.compose.material3.SearchBarDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.Stable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.composed
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusEvent
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.lifecycle.ViewModelStoreOwner
import androidx.lifecycle.viewmodel.compose.viewModel
import dev.vndx.flashbang.R
import dev.vndx.flashbang.World
import dev.vndx.flashbang.ui.CardTreeUiState
import dev.vndx.flashbang.ui.CardTreeViewModel
import dev.vndx.flashbang.ui.ShimmerProvider
import dev.vndx.flashbang.ui.Sizes
import dev.vndx.flashbang.ui.TagInfo
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.withContext
import kotlinx.serialization.Serializable
import uniffi.mobile.CardHandle
import uniffi.mobile.FuzzyStatus
import uniffi.mobile.Tag
import kotlin.math.exp

// Fake card names for skeleton / shimmer loading
val FakeCards = listOf(
    "Mitochondria",
    "Sun tzu: The art of war",
    "I need at  least two more of these don't I ?",
    "Lorem ipsum dolor sit amet",
    "I'm all out of ideas"
)

fun pollFuzzyFlow(world: World) = flow {
    var run = true
    while (run) {
        val status = withContext(Dispatchers.IO) { world.fuzzyTick() }
        if (status != FuzzyStatus.STALE) {
            emit(world.fuzzyResults().map { world.cards()[it.toInt()] })

            run = status != FuzzyStatus.FINISHED
        }

        delay(50)
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Stable
fun Modifier.clearFocusOnKeyboardDismiss(): Modifier = composed {
    var isFocused by remember { mutableStateOf(false) }
    var keyboardAppearedSinceLastFocused by remember { mutableStateOf(false) }

    if (isFocused) {
        val imeIsVisible = WindowInsets.ime.getBottom(LocalDensity.current) > 0
        val focusManager = LocalFocusManager.current

        LaunchedEffect(imeIsVisible) {
            if (imeIsVisible) {
                keyboardAppearedSinceLastFocused = true
            } else if (keyboardAppearedSinceLastFocused) {
                focusManager.clearFocus()
            }
        }
    }

    onFocusEvent {
        if (isFocused != it.isFocused) {
            isFocused = it.isFocused
            if (isFocused) keyboardAppearedSinceLastFocused = false
        }
    }
}

@Serializable
open class ExploreScreen() : Screen {
    // Should be val but https://youtrack.jetbrains.com/issue/KT-38958
    var root: Tag? = null
    override fun tab(): Tab = Tab.Cards
    override fun showTabs(): Boolean = root == null

    // Workaround for serialization
    constructor(tag: Tag?) : this() {
        root = tag
    }

    open fun enter(tag: Tag): Screen = ExploreScreen(tag)

    @Composable
    open fun Flashcard(
        handle: CardHandle,
        name: String,
        scheduled: Boolean,
    ) {
        dev.vndx.flashbang.ui.Flashcard(
            name = name,
            scheduled = scheduled,
        )
    }

    @Composable
    open fun Directory(
        tag: TagInfo,
        onClick: () -> Unit
    ) {
        dev.vndx.flashbang.ui.Directory(
            name = tag.name,
            cards = tag.indirectCards.size,
            onClick = onClick
        )
    }

    @OptIn(ExperimentalMaterial3Api::class)
    @Composable
    override fun Compose(onNavigate: (Screen) -> Unit) {

        val keyboardController = LocalSoftwareKeyboardController.current
        val cardTreeViewModel =
            viewModel<CardTreeViewModel>(viewModelStoreOwner = LocalActivity.current as ViewModelStoreOwner)
        val world = cardTreeViewModel.world
        val state by cardTreeViewModel.uiState.collectAsState()
        var query by remember { mutableStateOf("") }

        val directories by remember(query) {
            derivedStateOf {
                (root?.let { state.tags[it.toInt()].children }
                    ?: state.rootTags).map { tag -> state.tags[tag.toInt()] }
            }
        }

        val cards by remember(query) {
            derivedStateOf {
                (root?.let { state.tags[it.toInt()].cards }
                    ?: emptyList()).map { handle -> state.cards[handle.toInt()] }
            }
        }

        val searchResults by remember(query) {
            if (query.isNotEmpty()) {
                pollFuzzyFlow(world)
            } else {
                emptyFlow()
            }
        }.collectAsState(emptyList())

        Column(
            modifier = Modifier.Companion.fillMaxSize(),
            horizontalAlignment = Alignment.Companion.CenterHorizontally
        ) {
            SearchBar(
                modifier = Modifier.Companion
                    .fillMaxWidth()
                    .padding(Sizes.spacingMedium, 0.dp),
                windowInsets = WindowInsets(top = 0.dp),
                shape = RoundedCornerShape(Sizes.cornerRadiusLarge),
                inputField = {

                    SearchBarDefaults.InputField(
                        modifier = Modifier.clearFocusOnKeyboardDismiss(),
                        query = query,
                        onQueryChange = {
                            world.fuzzyInit(query)
                            query = it
                        },
                        onSearch = {
                            keyboardController?.hide()
                        },
                        onExpandedChange = {},
                        expanded = false,
                        leadingIcon = {
                            Icon(
                                painter = painterResource(R.drawable.outline_search_24),
                                contentDescription = null,
                                tint = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        },
                        placeholder = {
                            Text(
                                stringResource(R.string.search),
                                style = MaterialTheme.typography.bodyLarge
                            )
                        },
                    )
                },
                expanded = false,
                onExpandedChange = {},
            ) { }
            when (state) {
                is CardTreeUiState.Loading -> {
                    ShimmerProvider() {
                        LazyColumn(
                            modifier = Modifier.Companion
                                .fillMaxSize()
                                .padding(0.dp, Sizes.spacingSmall, 0.dp)
                        ) {
                            items(FakeCards) { name ->
                                // We use this one here since I don't have a CardHandle to give
                                dev.vndx.flashbang.ui.Flashcard(
                                    name = name,
                                    scheduled = true,
                                )
                            }
                        }
                    }
                }

                else -> {
                    LazyColumn(
                        modifier = Modifier.Companion
                            .fillMaxSize()
                            .padding(0.dp, Sizes.spacingSmall, 0.dp)
                    ) {
                        if (query.isEmpty()) {
                            items(directories) { dirTag ->
                                Directory(
                                    tag = dirTag,
                                    onClick = { onNavigate(enter(dirTag.id)) })
                            }
                            items(cards) { card ->
                                Flashcard(
                                    handle = card.handle(),
                                    name = card.name(),
                                    scheduled = false,
                                )
                            }
                        } else {
                            items(searchResults) { card ->
                                Flashcard(
                                    handle = card.handle(),
                                    name = card.name(),
                                    scheduled = false,
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}

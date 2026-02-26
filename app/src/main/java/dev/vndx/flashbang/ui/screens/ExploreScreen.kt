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
import androidx.compose.material3.IconButton
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
import androidx.compose.ui.focus.onFocusEvent
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.lifecycle.ViewModelStoreOwner
import androidx.lifecycle.viewmodel.compose.viewModel
import dev.vndx.flashbang.domain.Card
import dev.vndx.flashbang.R
import dev.vndx.flashbang.domain.Tag
import dev.vndx.flashbang.ui.CardsUiState
import dev.vndx.flashbang.ui.CardsViewModel
import dev.vndx.flashbang.ui.ExploreViewModel
import dev.vndx.flashbang.ui.ShimmerProvider
import dev.vndx.flashbang.ui.Sizes
import kotlinx.serialization.Serializable
import kotlinx.serialization.Transient

// Fake card names for skeleton / shimmer loading
val FakeCards = listOf(
    "Mitochondria",
    "Sun tzu: The art of war",
    "I need at least two more of these don't I ?",
    "Lorem ipsum dolor sit amet",
    "I'm all out of ideas"
)

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
open class ExploreScreen(var savedQuery: String = "") : Screen {
    // Should be val but https://youtrack.jetbrains.com/issue/KT-38958
    @Transient
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
        card: Card,
        scheduled: Boolean,
        onNavigate: (Screen) -> Unit
    ) {
        dev.vndx.flashbang.ui.Flashcard(
            name = card.name,
            scheduled = scheduled,
            onClick = {
                onNavigate(CardPreviewScreen(card.id))
            },
            onLongClick = {
                onNavigate(CardHistoryScreen(card.id))
            }
        )
    }

    @Composable
    open fun Directory(
        tag: Tag, onClick: () -> Unit
    ) {
        dev.vndx.flashbang.ui.Directory(
            name = tag.name, cards = tag.indirectCards.size, onClick = onClick
        )
    }

    @Composable
    override fun ComposeTopBarAction(onNavigate: (Screen) -> Unit, onBack: (Int?) -> Unit) {
        IconButton(onClick = {
            onBack(root?.ancestors?.size?.let { it + 1 })
        }) {
            Icon(
                painter = painterResource(R.drawable.outline_home_32),
                contentDescription = null,
                tint = MaterialTheme.colorScheme.onBackground
            )
        }
    }

    @OptIn(ExperimentalMaterial3Api::class)
    @Composable
    override fun Compose(onNavigate: (Screen) -> Unit, onBack: (Int?) -> Unit) {

        val keyboardController = LocalSoftwareKeyboardController.current
        val cardsViewModel =
            viewModel<CardsViewModel>(viewModelStoreOwner = LocalActivity.current as ViewModelStoreOwner)
        val exploreViewModel = viewModel<ExploreViewModel>()

        val state by cardsViewModel.uiState.collectAsState()

        // Initialize local query state from the saved property
        var query by remember { mutableStateOf(savedQuery) }

        // Update the viewmodel when the screen enters composition
        LaunchedEffect(Unit) {
            exploreViewModel.onSearchQueryChanged(savedQuery)
        }

        val directories by remember(query) {
            derivedStateOf {
                (root?.children?.toList() ?: state.rootTags)
            }
        }

        val cards by remember(query) {
            derivedStateOf {
                (root?.cards?.toList() ?: emptyList())
            }
        }

        val searchResultIds by exploreViewModel.searchResultIds.collectAsState()

        val searchResults by remember(searchResultIds, state) {
            derivedStateOf {
                searchResultIds.mapNotNull { state.cards[it] }
            }
        }

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
                        onQueryChange = { newQuery ->
                            query = newQuery
                            savedQuery = newQuery // Persist state in the Screen object
                            exploreViewModel.onSearchQueryChanged(newQuery)
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
                is CardsUiState.Loading -> {
                    ShimmerProvider() {
                        LazyColumn(
                            modifier = Modifier.Companion
                                .fillMaxSize()
                                .padding(0.dp, Sizes.spacingSmall, 0.dp)
                        ) {
                            items(FakeCards, key = { it }) { name ->
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
                            items(directories, key = { "tag:${it.fullPath}" }) { dirTag ->
                                Directory(
                                    tag = dirTag, onClick = { onNavigate(enter(dirTag)) })
                            }
                            items(cards, key = { "card:${it.id}" }) { card ->
                                Flashcard(
                                    card = card,
                                    scheduled = false,
                                    onNavigate = onNavigate
                                )
                            }
                        } else {
                            items(searchResults, key = { it.id }) { card ->
                                Flashcard(
                                    card = card,
                                    scheduled = false,
                                    onNavigate = onNavigate
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}

package dev.vndx.flashbang.ui.screens

import androidx.activity.compose.LocalActivity
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Checkbox
import androidx.compose.material3.LocalMinimumInteractiveComponentSize
import androidx.compose.material3.TriStateCheckbox
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateSetOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.state.ToggleableState
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelStoreOwner
import androidx.lifecycle.viewmodel.compose.viewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.vndx.flashbang.domain.Card
import dev.vndx.flashbang.domain.Tag
import dev.vndx.flashbang.ui.Directory
import dev.vndx.flashbang.ui.Flashcard
import dev.vndx.flashbang.ui.Sizes
import kotlinx.serialization.Serializable
import kotlinx.serialization.Transient
import javax.inject.Inject

@HiltViewModel
class SelectionViewModel @Inject constructor() : ViewModel() {
    val selection = mutableStateSetOf<Card>()

    fun isSelected(card: Card): Boolean =
        selection.contains(card)

    fun isEmpty(): Boolean = selection.isEmpty()

    fun clear() = selection.clear()

    fun toggleCard(card: Card) {
        if (!selection.add(card)) {
            selection.remove(card)
        }
    }

    fun deselectCard(card: Card) {
        selection.remove(card)
    }

    fun selectCard(card: Card) {
        selection.add(card)
    }
}

@Serializable
class SelectionScreen(@Transient val _selection: SelectionViewModel? = null) : ExploreScreen() {
    override fun tab(): Tab = Tab.Study

    override fun showTabs(): Boolean = false

    constructor(tag: Tag?, _selection: SelectionViewModel? = null) : this(_selection) {
        root = tag
    }

    @get:Composable
    val selection: SelectionViewModel get() = _selection ?: viewModel()

    override fun enter(tag: Tag): Screen = SelectionScreen(tag, _selection)

    @Composable
    override fun Flashcard(
        card: Card,
        scheduled: Boolean,
        onNavigate: (Screen) -> Unit,
    ) {
        // selection is an @Composable getter, so needs to be called in this scope
        val selection = selection
        val selected = selection.isSelected(card)

        Flashcard(
            name = card.name(),
            scheduled = false,
            onClick = { selection.toggleCard(card) },
            onLongClick = { onNavigate(CardPreviewScreen(card.id)) }
        ) {

            CompositionLocalProvider(LocalMinimumInteractiveComponentSize provides Dp.Companion.Unspecified) {
                Checkbox(
                    checked = selected,
                    onCheckedChange = { selection.toggleCard(card) },
                    modifier = Modifier
                        .align(Alignment.Companion.Top)
                        .padding(0.dp, Sizes.spacingLarge, 0.dp)
                )
            }
        }
    }

    @Composable
    override fun Directory(tag: Tag, onClick: () -> Unit) {
        val selection = selection

        val state by remember {
            derivedStateOf {
                when {
                    tag.indirectCards.all { selection.isSelected(it) } -> ToggleableState.On
                    tag.indirectCards.none { selection.isSelected(it) } -> ToggleableState.Off
                    else -> ToggleableState.Indeterminate
                }
            }
        }

        Directory(
            name = tag.name,
            cards = tag.indirectCards.size,
            onClick = onClick
        ) {
            CompositionLocalProvider(LocalMinimumInteractiveComponentSize provides Dp.Companion.Unspecified) {
                TriStateCheckbox(
                    state = state,
                    onClick = {
                        when (state) {
                            ToggleableState.Off -> tag.indirectCards.forEach {
                                selection.selectCard(
                                    it
                                )
                            }

                            else -> tag.indirectCards.forEach { selection.deselectCard(it) }
                        }
                    },
                    modifier = Modifier
                        .align(Alignment.Companion.Top)
                        .padding(0.dp, Sizes.spacingLarge, 0.dp)
                )
            }
        }
    }
}

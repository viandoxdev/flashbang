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
import dev.vndx.flashbang.ui.Directory
import dev.vndx.flashbang.ui.Flashcard
import dev.vndx.flashbang.ui.Sizes
import dev.vndx.flashbang.ui.TagInfo
import kotlinx.serialization.Serializable
import uniffi.mobile.CardHandle
import uniffi.mobile.Tag
import javax.inject.Inject

@HiltViewModel
class SelectionViewModel @Inject constructor() : ViewModel() {
    private val selection = mutableStateSetOf<CardHandle>()

    fun isSelected(card: CardHandle): Boolean =
        selection.contains(card)

    fun toggleCard(card: CardHandle) {
        if (!selection.add(card)) {
            selection.remove(card)
        }
    }

    fun deselectCard(card: CardHandle) {
        selection.remove(card)
    }

    fun selectCard(card: CardHandle) {
        selection.add(card)
    }
}

@Serializable
class SelectionScreen() : ExploreScreen() {
    override fun tab(): Tab = Tab.Study

    override fun showTabs(): Boolean = false

    constructor(tag: Tag?) : this() {
        root = tag
    }

    override fun enter(tag: Tag): Screen = SelectionScreen(tag)

    @get:Composable
    private val selection: SelectionViewModel
        get() = viewModel<SelectionViewModel>(viewModelStoreOwner = LocalActivity.current as ViewModelStoreOwner)

    @Composable
    override fun Flashcard(
        handle: CardHandle,
        name: String,
        scheduled: Boolean,
    ) {
        // selection is an @Composable getter, so needs to be called in this scope
        val selection = selection
        val selected = selection.isSelected(handle)

        Flashcard(
            name = name,
            scheduled = false,
            onClick = { selection.toggleCard(handle) }
        ) {

            CompositionLocalProvider(LocalMinimumInteractiveComponentSize provides Dp.Companion.Unspecified) {
                Checkbox(
                    checked = selected,
                    onCheckedChange = { selection.toggleCard(handle) },
                    modifier = Modifier
                        .align(Alignment.Companion.Top)
                        .padding(0.dp, Sizes.spacingLarge, 0.dp)
                )
            }
        }
    }

    @Composable
    override fun Directory(tag: TagInfo, onClick: () -> Unit) {
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

package dev.vndx.flashbang.ui

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.vndx.flashbang.domain.Card
import dev.vndx.flashbang.TAG
import dev.vndx.flashbang.domain.Tag
import dev.vndx.flashbang.Core
import dev.vndx.flashbang.domain.Header
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.stateIn
import uniffi.mobile.CoreException
import uniffi.mobile.LoadResult
import javax.inject.Inject

data class CardRepositoryDetails(
    val repository: String,
    val branch: String,
    val token: String?,
)


data class CardsData(
    val cards: Map<String, Card>, val rootTags: List<Tag>
) {
    companion object {
        fun fromLoad(core: Core, load: LoadResult): CardsData {
            core.core.fuzzyReset()

            val tagsMap = mutableMapOf<String, Tag>()
            val rootTags = mutableSetOf<Tag>()
            fun tagOf(fullPath: String): Tag = tagsMap.getOrPut(fullPath) {
                val ancestors =
                    fullPath.withIndex().filter { (_, ch) -> ch == '.' }.map { (index, _) ->
                        tagOf(fullPath.slice(0..(index - 1)))
                    }
                val tag = Tag(fullPath, ancestors)

                rootTags.add(tag.root)

                tag
            }

            val cards = load.cards.map {
                val locations = it.locations.map { path -> tagOf(path) }
                // TODO: Get rid of the whole header id thing on rust side, its unnecessary
                val card = Card(
                    it.id,
                    it.name,
                    locations,
                    it.question,
                    it.answer,
                    it.header?.let { headerInfo ->
                        Header(
                            headerInfo.content()
                        )
                    })

                locations.forEach { tag ->
                    tag.ancestors.forEach { ancestor ->
                        ancestor.addCardIndirect(card)
                    }
                    tag.addCard(card)
                }

                card
            }

            core.core.fuzzyAddItems(cards)

            return CardsData(cards.associateBy { it.id }, rootTags.toList())
        }
    }
}

fun loadCardsFlow(core: Core, repo: String, branch: String, token: String?) = flow {
    val result = try {
        Log.w("Flashbang", "Hello, I am about to start something that might take long.")
        core.loadFromGithub(repo, branch, token)
    } catch (e: CoreException) {
        Log.e("Flashbang", "$e")

        emit(CardsUiState.Failure(e))
        return@flow
    }

    emit(CardsUiState.Success(CardsData.fromLoad(core, result)))
}.flowOn(Dispatchers.IO)

@HiltViewModel
class CardsViewModel @Inject constructor(
    val core: Core,
) : ViewModel() {
    private val loadTrigger =
        MutableSharedFlow<CardRepositoryDetails>(extraBufferCapacity = 1, replay = 1)

    @OptIn(ExperimentalCoroutinesApi::class)
    val uiState: StateFlow<CardsUiState> = loadTrigger.flatMapLatest {
        Log.w(TAG, "Got load details")
        loadCardsFlow(core, it.repository, it.branch, it.token)
    }.stateIn(
        scope = viewModelScope,
        initialValue = CardsUiState.Loading,
        started = SharingStarted.WhileSubscribed(5_000)
    )

    fun load(details: CardRepositoryDetails) {
        loadTrigger.tryEmit(details)
    }
}

sealed interface CardsUiState {
    data object Loading : CardsUiState

    data class Success(
        val data: CardsData
    ) : CardsUiState {
        override val cards: Map<String, Card>
            get() = data.cards
        override val rootTags: List<Tag>
            get() = data.rootTags
    }

    data class Failure(val exception: CoreException) : CardsUiState

    val cards: Map<String, Card> get() = emptyMap()
    val rootTags: List<Tag> get() = emptyList()
}

package dev.vndx.flashbang.ui

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.vndx.flashbang.TAG
import dev.vndx.flashbang.World
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.stateIn
import uniffi.mobile.AnyException
import uniffi.mobile.Card
import uniffi.mobile.CardHandle
import uniffi.mobile.Tag
import javax.inject.Inject
import kotlin.collections.zipWithNext

data class CardRepositoryDetails(
    val repository: String,
    val branch: String,
    val token: String?,
)

class TagInfoDraft(val id: Tag, val name: String) {
    var parent: Tag? = null
    private var children = mutableSetOf<Tag>()
    private var cards = mutableSetOf<CardHandle>()
    private var indirectCards = mutableSetOf<CardHandle>()

    fun addCard(card: CardHandle) {
        indirectCards.add(card)
        cards.add(card)
    }

    fun addCardIndirect(card: CardHandle) = indirectCards.add(card)
    fun addChild(child: Tag) = children.add(child)

    fun build(): TagInfo = TagInfo(
        parent = parent,
        id = id,
        name = name,
        children = children.toList(),
        cards = cards.toList(),
        indirectCards = indirectCards.toList()
    )
}

data class TagInfo(
    val parent: Tag?,
    val id: Tag,
    val name: String,
    val children: List<Tag>,
    val indirectCards: List<CardHandle>,
    val cards: List<CardHandle>,
)

data class CardTreeData(
    val cards: List<Card>,
    val tags: List<TagInfo>,
    val rootTags: List<Tag>
)

fun loadCardsFlow(world: World, repo: String, branch: String, token: String?) = flow {
    try {
        Log.w("Flashbang", "Hello, I am about to start something that might take long.")
        val errors = world.loadFromGithub(repo, branch, token)
    } catch (e: AnyException) {
        Log.e("Flashbang", "${e.display()} ${e.debug()}")

        emit(CardTreeUiState.Failure(e))
        return@flow
    }

    val cards = world.cards()
    val tagNames = world.tagNames()

    val newTags = tagNames.mapIndexed { index, name ->
        TagInfoDraft(
            id = index.toULong(),
            name = name,
        )
    }

    val root = TagInfoDraft(
        id = 0uL,
        name = "",
    )

    for (card in cards) {
        val handle = card.handle()
        val paths = card.paths()

        for (path in paths) {
            for ((parent, child) in path.zipWithNext()) {
                newTags[child.toInt()].parent = parent
                newTags[parent.toInt()].addChild(child)
                newTags[parent.toInt()].addCardIndirect(handle)
            }

            newTags[path.last().toInt()].addCard(handle)
            root.addChild(path.first())
        }
    }

    val tags = newTags.map { it.build() }
    val rootTags = root.build().children

    emit(CardTreeUiState.Success(CardTreeData(cards, tags, rootTags)))
}.flowOn(Dispatchers.IO)

@HiltViewModel
class CardTreeViewModel @Inject constructor(
    val world: World,
) : ViewModel() {
    private val loadTrigger = MutableSharedFlow<CardRepositoryDetails>(extraBufferCapacity = 1, replay = 1)

    init {
        Log.w(TAG, "Hello !")
    }

    @OptIn(ExperimentalCoroutinesApi::class)
    val uiState: StateFlow<CardTreeUiState> =
        loadTrigger
        .flatMapLatest {
            Log.w(TAG, "Got load details")
            loadCardsFlow(world,it.repository, it.branch, it.token)
        }
            .stateIn(
                scope = viewModelScope,
                initialValue = CardTreeUiState.Loading,
                started = SharingStarted.WhileSubscribed(5_000)
            )

    fun load(details: CardRepositoryDetails) {
        loadTrigger.tryEmit(details)
    }
}

sealed interface CardTreeUiState {
    data object Loading : CardTreeUiState

    data class Success(
        val data: CardTreeData
    ) : CardTreeUiState {
        override val cards: List<Card>
            get() = data.cards
        override val tags: List<TagInfo>
            get() = data.tags
        override val rootTags: List<Tag>
            get() = data.rootTags
    }

    data class Failure(val exception: AnyException) : CardTreeUiState

    val cards: List<Card> get() = emptyList()
    val tags: List<TagInfo> get() = emptyList()
    val rootTags: List<Tag> get() = emptyList()
}

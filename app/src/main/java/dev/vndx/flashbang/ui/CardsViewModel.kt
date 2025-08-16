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
import uniffi.mobile.Tag
import javax.inject.Inject

data class CardRepositoryDetails(
    val repository: String,
    val branch: String,
    val token: String?,
)


data class CardsData(
    val cards: List<Card>,
    val rootTags: List<Tag>
)

fun loadCardsFlow(world: World, repo: String, branch: String, token: String?) = flow {
    try {
        Log.w("Flashbang", "Hello, I am about to start something that might take long.")
        val errors = world.loadFromGithub(repo, branch, token)
    } catch (e: AnyException) {
        Log.e("Flashbang", "${e.display()} ${e.debug()}")

        emit(CardsUiState.Failure(e))
        return@flow
    }

    emit(CardsUiState.Success(CardsData(world.cards(), world.roots())))
}.flowOn(Dispatchers.IO)

@HiltViewModel
class CardsViewModel @Inject constructor(
    val world: World,
) : ViewModel() {
    private val loadTrigger = MutableSharedFlow<CardRepositoryDetails>(extraBufferCapacity = 1, replay = 1)

    init {
        Log.w(TAG, "Hello !")
    }

    @OptIn(ExperimentalCoroutinesApi::class)
    val uiState: StateFlow<CardsUiState> =
        loadTrigger
        .flatMapLatest {
            Log.w(TAG, "Got load details")
            loadCardsFlow(world,it.repository, it.branch, it.token)
        }
            .stateIn(
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
        override val cards: List<Card>
            get() = data.cards
        override val rootTags: List<Tag>
            get() = data.rootTags
    }

    data class Failure(val exception: AnyException) : CardsUiState

    val cards: List<Card> get() = emptyList()
    val rootTags: List<Tag> get() = emptyList()
}

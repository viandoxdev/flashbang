package dev.vndx.flashbang.ui

import androidx.datastore.core.DataStore
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.vndx.flashbang.CardReview
import dev.vndx.flashbang.Studies
import dev.vndx.flashbang.Core
import dev.vndx.flashbang.Rating
import dev.vndx.flashbang.data.toProto
import dev.vndx.flashbang.domain.Card
import dev.vndx.flashbang.domain.Study
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.mobile.SchedulerMemoryState
import java.time.LocalDateTime
import java.time.ZoneOffset
import java.time.temporal.ChronoUnit
import java.time.temporal.TemporalUnit
import javax.inject.Inject

@HiltViewModel
class StudiesViewModel @Inject constructor(
    private val dataStore: DataStore<Studies>,
    private val core: Core,
) : ViewModel() {
    val studiesState = dataStore.data.map { state ->
        val collect = studies.keys

        ids = state.ids
        state.studiesMap.values.forEach { study ->
            collect.remove(study.id)
            studies.put(study.id, Study.fromProto(study))
        }

        for (id in collect) {
            studies.remove(id)
        }

        StudiesState.Success(state)
    }.stateIn(
        viewModelScope, SharingStarted.WhileSubscribed(5000), StudiesState.Loading
    )

    var ids = 1L
    val studies = mutableMapOf<Long, Study>()

    private fun edit(transform: Studies.Builder.() -> Unit) {
        viewModelScope.launch {
            dataStore.updateData { studies -> studies.toBuilder().apply(transform).build() }
        }
    }

    private fun editStudy(study: Study, transform: dev.vndx.flashbang.Study.Builder.() -> Unit) {
        edit {
            val id = study.id
            val value = getStudiesOrThrow(id).toBuilder().apply(transform).build()
            putStudies(id, value)
        }
    }

    fun createStudy(selection: List<String>, name: String) {
        val id = ids

        ids += 1

        val study = Study(id, LocalDateTime.now(), selection, name, mutableMapOf(), false)

        edit {
            putStudies(id, study.toProto())
        }
    }

    fun finalizeStudy(study: Study) {
        study.finished = true

        editStudy(study) {
            setFinished(true)
        }
    }

    fun renameStudy(study: Study, name: String) {
        study.name = name
        editStudy(study) {
            setName(name)
        }
    }

    fun updateStudy(study: Study, rating: Rating, card: Card) {
        study.reviews.put(card.id, rating)

        // TODO: update FSRS Memory state:
        //       Update rust side, which will give us new memory state
        //       and feed into proto

        edit {
            val protoStudy = getStudiesOrThrow(study.id)
            protoStudy.reviewsMap.put(card.id, rating)

            putStudies(study.id, protoStudy)

            val memoryState = getMemoryOrThrow(card.id)
            val lastReview = memoryState.reviewsList.lastOrNull()?.timestamp?.let {
                LocalDateTime.ofEpochSecond(
                    it, 0, ZoneOffset.UTC
                )
            } ?: LocalDateTime.now()
            memoryState.reviewsList.add(
                CardReview.newBuilder().setTimestamp(
                    LocalDateTime.now().toEpochSecond(
                        ZoneOffset.UTC
                    )
                ).setRating(rating).build()
            )
            val daysElapsed = lastReview.until(LocalDateTime.now(), ChronoUnit.DAYS)

            val nextState = core.core.schedulerNextState(
                SchedulerMemoryState(
                    memoryState.stability,
                    memoryState.difficulty,
                ), daysElapsed.toUInt()
            ).let {
                when (rating) {
                    Rating.RATING_HARD -> it.hard
                    Rating.RATING_GOOD -> it.good
                    Rating.RATING_EASY -> it.easy
                    else -> it.again
                }
            }

            val scheduledFor = LocalDateTime.now().plusHours((nextState.delay * 24f).toLong())

            putMemory(
                card.id,
                memoryState.toBuilder().setStability(nextState.state.stability)
                    .setDifficulty(nextState.state.difficulty).build()
            )
        }
    }
}

sealed interface StudiesState {
    data object Loading : StudiesState

    data class Success(private val inner: Studies) : StudiesState {
        override val studies: Studies get() = inner
    }

    val studies: Studies get() = throw IllegalStateException("Studies not loaded yet")
}

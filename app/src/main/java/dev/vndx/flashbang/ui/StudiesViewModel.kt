package dev.vndx.flashbang.ui

import androidx.datastore.core.DataStore
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.vndx.flashbang.CardReview
import dev.vndx.flashbang.Studies
import dev.vndx.flashbang.Core
import dev.vndx.flashbang.Rating
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
import javax.inject.Inject

@HiltViewModel
class StudiesViewModel @Inject constructor(
    private val dataStore: DataStore<Studies>,
    private val core: Core,
) : ViewModel() {
    val studiesState = dataStore.data.map { state ->
        StudiesState.Success(state)
    }.stateIn(
        viewModelScope, SharingStarted.WhileSubscribed(5000), StudiesState.Loading
    )


    private fun edit(transform: Studies.Builder.() -> Unit) {
        viewModelScope.launch {
            dataStore.updateData { studies -> studies.toBuilder().apply(transform).build() }
        }
    }

    private fun editStudy(study: Study, transform: dev.vndx.flashbang.Study.Builder.() -> Unit) {
        edit {
            val value = getStudiesOrThrow(study.id).toBuilder().apply(transform).build()
            putStudies(study.id, value)
        }
    }

    fun createStudy(selection: List<String>, name: String) {
        val id = when (studiesState.value) {
            is StudiesState.Success -> studiesState.value.proto.ids
            else -> throw IllegalStateException("Can't create new study when StudyState hasn't loaded successfully")
        }
        
        val study = Study(id, LocalDateTime.now(), selection, name, mutableMapOf(), false)

        edit {
            putStudies(id, study.toProto())
            setIds(id + 1)
        }
    }

    fun deleteStudy(study: Study) {
        edit {
            removeStudies(study.id)
        }
    }

    fun finalizeStudy(study: Study) {
        editStudy(study) {
            setFinished(true)
        }
    }

    fun renameStudy(study: Study, name: String) {
        editStudy(study) {
            setName(name)
        }
    }

    fun updateStudy(study: Study, rating: Rating, card: Card) {
        edit {
            // Update study
            val protoStudy = getStudiesOrThrow(study.id)
            protoStudy.reviewsMap.put(card.id, rating)
            putStudies(study.id, protoStudy)

            // Update memory state

            // Fetch current state
            val memoryState = getMemoryOrThrow(card.id)
            val lastReview = memoryState.reviewsList.lastOrNull()?.timestamp?.let {
                LocalDateTime.ofEpochSecond(
                    it, 0, ZoneOffset.UTC
                )
            } ?: LocalDateTime.now()

            // Add latest review (for parameters computation
            memoryState.reviewsList.add(
                CardReview.newBuilder().setTimestamp(
                    LocalDateTime.now().toEpochSecond(
                        ZoneOffset.UTC
                    )
                ).setRating(rating).build()
            )

            // Update memory state
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

            // Update card's next scheduled date
            val scheduledFor = LocalDateTime.now().plusHours((nextState.delay * 24f).toLong())
            card.scheduledFor = scheduledFor.toLocalDate()

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
        private val _studies = proto.studiesMap.mapValues { Study.fromProto(it.value) }
        override val proto: Studies get() = inner
        override val studies: Map<Long, Study>
            get() = _studies
    }

    val proto: Studies get() = throw IllegalStateException("Studies not loaded yet")
    val studies: Map<Long, Study>
        get() = emptyMap()
}

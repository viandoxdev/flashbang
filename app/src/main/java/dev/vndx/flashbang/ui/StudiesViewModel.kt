package dev.vndx.flashbang.ui

import android.util.Log
import androidx.datastore.core.DataStore
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.vndx.flashbang.CardMemoryState
import dev.vndx.flashbang.CardReview
import dev.vndx.flashbang.Studies
import dev.vndx.flashbang.Core
import dev.vndx.flashbang.Rating
import dev.vndx.flashbang.TAG
import dev.vndx.flashbang.domain.Card
import dev.vndx.flashbang.domain.Study
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.first
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

    init {
        viewModelScope.launch(Dispatchers.IO) {
            studiesState.filterIsInstance<StudiesState.Success>().first().let { state ->
                val memoryMap = state.proto.memoryMap
                val toUpdate = memoryMap.filter { (id, memory) ->
                    memory.dueDate == 0L && memory.reviewsList.isNotEmpty()
                }

                if (toUpdate.isNotEmpty()) {
                    Log.i(TAG, "Migrating ${toUpdate.size} cards to have due_date")
                    val updates = mutableMapOf<String, CardMemoryState>()

                    toUpdate.forEach { (id, memory) ->
                        val (currentState, lastDelay, lastReviewTime) = calculateStateFromReviews(
                            id,
                            memory.reviewsList
                        ) ?: return@forEach

                        val dueDate = lastReviewTime!!.plusHours((lastDelay * 24f).toLong())
                            .toEpochSecond(ZoneOffset.UTC)

                        val newMemory = memory.toBuilder()
                            .setStability(currentState?.stability ?: 0f)
                            .setDifficulty(currentState?.difficulty ?: 0f)
                            .setDueDate(dueDate)
                            .build()
                        updates[id] = newMemory
                    }

                    if (updates.isNotEmpty()) {
                        edit {
                            updates.forEach { (id, mem) ->
                                putMemory(id, mem)
                            }
                        }
                    }
                }
            }
        }
    }


    private fun edit(transform: Studies.Builder.() -> Unit) {
        viewModelScope.launch {
            dataStore.updateData { studies -> studies.toBuilder().apply(transform).build() }
            Log.w(TAG, "Finished writing studies")
        }
    }

    private fun editStudy(study: Study, transform: dev.vndx.flashbang.Study.Builder.() -> Unit) {
        edit {
            val value = getStudiesOrThrow(study.id).toBuilder().apply(transform).build()
            putStudies(study.id, value)
        }
    }

    fun createStudy(selection: List<String>, name: String): Study {
        val id = when (studiesState.value) {
            is StudiesState.Success -> studiesState.value.proto.ids
            else -> throw IllegalStateException("Can't create new study when StudyState hasn't loaded successfully")
        }

        val study = Study(id, LocalDateTime.now(), selection, name, mutableMapOf(), false)

        edit {
            putStudies(id, study.toProto())
            setIds(id + 1)
        }

        return study
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
        study.reviews.put(card.id, rating)

        edit {
            // Update study
            val protoStudy = getStudiesOrThrow(study.id)
            val newStudy = protoStudy.toBuilder().putReviews(card.id, rating).build()
            putStudies(study.id, newStudy)

            Log.w(TAG, "Updated study proto")

            // Fetch current state
            val memoryState = memoryMap[card.id]

            val lastReview = memoryState?.reviewsList?.lastOrNull()?.timestamp?.let {
                LocalDateTime.ofEpochSecond(
                    it, 0, ZoneOffset.UTC
                )
            } ?: LocalDateTime.now()

            // Update memory state
            val daysElapsed = lastReview.until(LocalDateTime.now(), ChronoUnit.DAYS)
            val nextState = core.core.schedulerNextState(
                memoryState?.let {
                    SchedulerMemoryState(
                        it.stability,
                        it.difficulty,
                    )
                }, daysElapsed.toUInt()
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
            val dueDate = scheduledFor.toEpochSecond(ZoneOffset.UTC)

            putMemory(
                card.id,
                (memoryState?.toBuilder()
                    ?: CardMemoryState.newBuilder()).setStability(nextState.state.stability)
                    .setDifficulty(nextState.state.difficulty)
                    .setDueDate(dueDate)
                    .addReviews(
                        CardReview.newBuilder().setTimestamp(
                            LocalDateTime.now().toEpochSecond(
                                ZoneOffset.UTC
                            )
                        ).setRating(rating).build()
                    ).build()
            )
        }
    }

    fun clearCardHistory(card: Card?, cardId: String) {
        viewModelScope.launch(Dispatchers.IO) {
            recalculateState(cardId, emptyList(), card)
        }
    }

    fun deleteCardReview(card: Card?, cardId: String, reviewTimestamp: Long) {
        val state = studiesState.value
        if (state !is StudiesState.Success) return

        val memoryState = state.proto.memoryMap[cardId] ?: return
        val reviews = memoryState.reviewsList.filter { it.timestamp != reviewTimestamp }
        viewModelScope.launch(Dispatchers.IO) {
            recalculateState(cardId, reviews, card)
        }
    }

    private fun recalculateState(cardId: String, reviews: List<CardReview>, card: Card?) {
        // Sort reviews by timestamp
        val sortedReviews = reviews.sortedBy { it.timestamp }

        val (currentState, lastDelay, lastReviewTime) = calculateStateFromReviews(
            cardId,
            sortedReviews
        ) ?: Triple(null, 0f, null)

        edit {
            if (sortedReviews.isEmpty()) {
                removeMemory(cardId)
            } else {
                val builder = if (containsMemory(cardId)) getMemoryOrThrow(cardId).toBuilder() else CardMemoryState.newBuilder()
                builder.clearReviews()
                builder.addAllReviews(sortedReviews)

                if (currentState != null) {
                    builder.setStability(currentState.stability)
                    builder.setDifficulty(currentState.difficulty)
                    if (lastReviewTime != null) {
                        val scheduledFor = lastReviewTime.plusHours((lastDelay * 24f).toLong())
                        builder.setDueDate(scheduledFor.toEpochSecond(ZoneOffset.UTC))
                    }
                } else {
                    // Reset if no reviews or cleared (though isEmpty check handles this usually)
                    builder.clearStability()
                    builder.clearDifficulty()
                    builder.clearDueDate()
                }

                putMemory(cardId, builder.build())
            }
        }

        // Update Card object if provided
        card?.let {
            if (lastReviewTime != null) {
                val scheduledFor = lastReviewTime!!.plusHours((lastDelay * 24f).toLong())
                it.scheduledFor = scheduledFor.toLocalDate()
            } else {
                it.scheduledFor = null
            }
        }
    }

    private fun calculateStateFromReviews(
        cardId: String,
        reviews: List<CardReview>
    ): Triple<SchedulerMemoryState?, Float, LocalDateTime?>? {
        val sortedReviews = reviews.sortedBy { it.timestamp }

        var currentState: SchedulerMemoryState? = null
        var lastReviewTime: LocalDateTime? = null
        var lastDelay: Float = 0f

        for (review in sortedReviews) {
            val currentReviewTime = LocalDateTime.ofEpochSecond(review.timestamp, 0, ZoneOffset.UTC)

            // Calculate days elapsed since last review
            // For the first review, daysElapsed is usually treated as 0 or time since creation.
            // Here we use 0 if it's the first review.
            val daysElapsed = lastReviewTime?.until(currentReviewTime, ChronoUnit.DAYS)?.toInt()
                ?.coerceAtLeast(0)?.toUInt() ?: 0u

            try {
                val nextStates = core.core.schedulerNextState(currentState, daysElapsed)

                val itemState = when (review.rating) {
                    Rating.RATING_HARD -> nextStates.hard
                    Rating.RATING_GOOD -> nextStates.good
                    Rating.RATING_EASY -> nextStates.easy
                    else -> nextStates.again
                }

                currentState = itemState.state
                lastDelay = itemState.delay
                lastReviewTime = currentReviewTime

            } catch (e: Exception) {
                Log.e(TAG, "Error calculating next state for card $cardId: $e")
                // If error, maybe stop replaying or continue with best effort?
                // For now, let's break to avoid inconsistent state accumulating
                break
            }
        }

        if (lastReviewTime == null) return null

        return Triple(currentState, lastDelay, lastReviewTime)
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

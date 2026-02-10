package dev.vndx.flashbang.ui

import androidx.datastore.core.DataStore
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.vndx.flashbang.Rating
import dev.vndx.flashbang.Studies
import dev.vndx.flashbang.CardMemoryState
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import java.time.LocalDate
import java.time.LocalDateTime
import java.time.ZoneId
import java.time.ZoneOffset
import javax.inject.Inject
import kotlin.math.roundToInt

data class StatisticsData(
    val today: TodayStats,
    val cardCounts: CardCounts,
    val reviewHistory: Map<LocalDate, ReviewDayStats>,
    val futureDue: Map<LocalDate, Int>,
    val stabilityDistribution: Map<Int, Int>,
    val difficultyDistribution: Map<Int, Int>
)

data class TodayStats(
    val learned: Int,
    val reviewed: Int,
    val relearned: Int,
    val againCount: Int,
    val passRate: Float,
    val totalReviews: Int
)

data class CardCounts(
    val young: Int,
    val mature: Int,
    val total: Int
)

data class ReviewDayStats(
    val total: Int,
    val again: Int,
    val hard: Int,
    val good: Int,
    val easy: Int
)

@HiltViewModel
class StatisticsViewModel @Inject constructor(
    dataStore: DataStore<Studies>
) : ViewModel() {

    val statisticsState: StateFlow<StatisticsState> = dataStore.data
        .map { studies ->
            val stats = calculateStatistics(studies)
            StatisticsState.Success(stats)
        }
        .stateIn(
            viewModelScope,
            SharingStarted.WhileSubscribed(5000),
            StatisticsState.Loading
        )

    private fun calculateStatistics(studies: Studies): StatisticsData {
        val memoryMap = studies.memoryMap
        val now = LocalDate.now()
        val startOfToday = now.atStartOfDay(ZoneId.systemDefault()).toEpochSecond()

        // Today's Stats
        var todayReviews = 0
        var todayAgain = 0
        var todayLearned = 0
        var todayReviewed = 0
        var todayRelearned = 0

        // History & Future
        val historyMap = mutableMapOf<LocalDate, ReviewDayStats>()
        val futureDueMap = mutableMapOf<LocalDate, Int>()
        val stabilityDist = mutableMapOf<Int, Int>()
        val difficultyDist = mutableMapOf<Int, Int>()

        // Counts
        var young = 0
        var mature = 0

        // Iterate over all cards in memory
        memoryMap.values.forEach { cardMemory ->
            val stability = cardMemory.stability
            val difficulty = cardMemory.difficulty

            // Stability Distribution (bucket by day)
            val stabilityBucket = stability.roundToInt()
            stabilityDist[stabilityBucket] = (stabilityDist[stabilityBucket] ?: 0) + 1

            // Difficulty Distribution (x10 for resolution, bucket 1-10 effectively)
            val difficultyBucket = (difficulty * 10).roundToInt()
            difficultyDist[difficultyBucket] = (difficultyDist[difficultyBucket] ?: 0) + 1

            // Young vs Mature (Interval >= 21 days)
            // Using stability as proxy for interval
            if (stability >= 21.0) {
                mature++
            } else {
                young++
            }

            // Iterate reviews
            cardMemory.reviewsList.forEachIndexed { index, review ->
                val reviewDate = LocalDateTime.ofEpochSecond(review.timestamp, 0, ZoneOffset.UTC)
                    .atZone(ZoneOffset.UTC)
                    .withZoneSameInstant(ZoneId.systemDefault())
                    .toLocalDate()

                // History
                val currentDayStats = historyMap.getOrPut(reviewDate) { ReviewDayStats(0, 0, 0, 0, 0) }
                val newStats = when (review.rating) {
                    Rating.RATING_AGAIN -> currentDayStats.copy(
                        total = currentDayStats.total + 1,
                        again = currentDayStats.again + 1
                    )
                    Rating.RATING_HARD -> currentDayStats.copy(
                        total = currentDayStats.total + 1,
                        hard = currentDayStats.hard + 1
                    )
                    Rating.RATING_GOOD -> currentDayStats.copy(
                        total = currentDayStats.total + 1,
                        good = currentDayStats.good + 1
                    )
                    Rating.RATING_EASY -> currentDayStats.copy(
                        total = currentDayStats.total + 1,
                        easy = currentDayStats.easy + 1
                    )
                    else -> currentDayStats
                }
                historyMap[reviewDate] = newStats

                // Today's Stats
                if (review.timestamp >= startOfToday) {
                    todayReviews++
                    if (review.rating == Rating.RATING_AGAIN) {
                        todayAgain++
                    }

                    if (index == 0) {
                        todayLearned++
                    } else {
                        if (review.rating == Rating.RATING_AGAIN) {
                            todayRelearned++
                        } else {
                            todayReviewed++
                        }
                    }
                }
            }

            // Future Due
            val lastReview = cardMemory.reviewsList.lastOrNull()
            if (lastReview != null) {
                val lastReviewDate = LocalDateTime.ofEpochSecond(lastReview.timestamp, 0, ZoneOffset.UTC)
                    .atZone(ZoneOffset.UTC)
                    .withZoneSameInstant(ZoneId.systemDefault())
                    .toLocalDate()
                val interval = stability.roundToInt().coerceAtLeast(1)
                val dueDate = lastReviewDate.plusDays(interval.toLong())

                // Only count future due
                if (dueDate.isAfter(now) || dueDate.isEqual(now)) {
                    futureDueMap[dueDate] = (futureDueMap[dueDate] ?: 0) + 1
                }
            }
        }

        val passRate = if (todayReviews > 0) {
            (todayReviews - todayAgain).toFloat() / todayReviews
        } else {
            0f
        }

        return StatisticsData(
            today = TodayStats(
                learned = todayLearned,
                reviewed = todayReviewed,
                relearned = todayRelearned,
                againCount = todayAgain,
                passRate = passRate,
                totalReviews = todayReviews
            ),
            cardCounts = CardCounts(young, mature, young + mature),
            reviewHistory = historyMap.toSortedMap(),
            futureDue = futureDueMap.toSortedMap(),
            stabilityDistribution = stabilityDist.toSortedMap(),
            difficultyDistribution = difficultyDist.toSortedMap()
        )
    }
}

sealed interface StatisticsState {
    data object Loading : StatisticsState
    data class Success(val data: StatisticsData) : StatisticsState
}

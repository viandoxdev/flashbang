package dev.vndx.flashbang.domain

import dev.vndx.flashbang.Rating
import dev.vndx.flashbang.ui.CardsUiState
import dev.vndx.flashbang.ui.LocalDateTimeSerializer
import kotlinx.serialization.Serializable
import java.time.LocalDateTime
import java.time.ZoneOffset

@Serializable
data class Study(
    val id: Long,
    @Serializable(with = LocalDateTimeSerializer::class)
    val timestamp: LocalDateTime,
    val selection: List<String>,
    val name: String,
    val reviews: MutableMap<String, Rating>,
    val finished: Boolean,
) {

    fun getOrBuildSelectionSummary(cards: CardsUiState): List<Item> {
        selectionSummary?.let {
            return it
        }

        val sum = Study.buildSelectionSummary(selection.mapNotNull { cards.cards[it] }.toSet())
        selectionSummary = sum
        return sum
    }

    private var selectionSummary: List<Item>? = null

    fun toProto(): dev.vndx.flashbang.Study = dev.vndx.flashbang.Study.newBuilder().setId(id)
        .setTimestamp(timestamp.toEpochSecond(ZoneOffset.UTC)).addAllSelection(selection)
        .setName(name).putAllReviews(reviews).setFinished(finished).build()

    companion object {
        fun buildSelectionSummary(selectedLeaves: Set<Card>): List<Item> {
            if (selectedLeaves.isEmpty()) return emptyList()

            fun collectCandidates(item: Item): List<Item> {
                val leaves = item.leafItems
                if (leaves.isNotEmpty() && selectedLeaves.containsAll(leaves)) {
                    return listOf(item)
                }
                return item.childItems.flatMap { collectCandidates(it) }
            }

            val candidates = selectedLeaves
                .flatMap { it.locations }
                .map { it.root }
                .distinct()
                .flatMap { collectCandidates(it) }
                .distinct()

            val sortedCandidates = candidates.sortedByDescending { it.leafItems.size }

            val coveredLeaves = mutableSetOf<Item>()
            val selectedItems = mutableListOf<Item>()

            for (candidate in sortedCandidates) {
                val leaves = candidate.leafItems
                if (leaves.any { !coveredLeaves.contains(it) }) {
                    selectedItems.add(candidate)
                    coveredLeaves.addAll(leaves)
                }
            }

            return selectedItems
        }

        fun fromProto(study: dev.vndx.flashbang.Study): Study = Study(
            study.id,
            LocalDateTime.ofEpochSecond(study.timestamp, 0, ZoneOffset.UTC),
            study.selectionList,
            study.name,
            study.reviewsMap.toMutableMap(),
            study.finished
        )
    }
}
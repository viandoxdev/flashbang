package dev.vndx.flashbang.domain

import dev.vndx.flashbang.Rating
import dev.vndx.flashbang.ui.CardsUiState
import kotlinx.serialization.Serializable
import java.time.LocalDateTime
import java.time.ZoneOffset

data class Study(
    val id: Long,
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
            // Set of the leaves that are contained in selectedItems
            val draftSelectedLeaves = mutableSetOf<Item>()
            val selectedItems = mutableSetOf<Item>()

            fun Item.walk() {
                if (selectedLeaves.containsAll(leafItems) && !draftSelectedLeaves.containsAll(
                        leafItems
                    )
                ) {
                    selectedItems.add(this)
                    draftSelectedLeaves.addAll(leafItems)
                } else {
                    childItems.forEach { it.walk() }
                }
            }

            for (root in selectedLeaves.flatMap { it.locations }.map { it.root }.toSet()) {
                root.walk()
            }

            return selectedItems.sortedBy { -it.leafItems.size }
        }

        fun fromProto(study: dev.vndx.flashbang.Study): Study = Study(
            study.id,
            LocalDateTime.ofEpochSecond(study.timestamp, 0, ZoneOffset.UTC),
            study.selectionList,
            study.name,
            study.reviewsMap,
            study.finished
        )
    }
}
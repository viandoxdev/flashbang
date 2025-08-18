package dev.vndx.flashbang.domain

import dev.vndx.flashbang.Rating
import java.time.LocalDateTime
import java.time.ZoneOffset

data class Study(
    val id: Long,
    val timestamp: LocalDateTime,
    val selection: List<String>,
    var name: String,
    val reviews: MutableMap<String, Rating>,
    var finished: Boolean,
) {
    fun toProto(): dev.vndx.flashbang.Study = dev.vndx.flashbang.Study.newBuilder()
        .setId(id)
        .setTimestamp(timestamp.toEpochSecond(ZoneOffset.UTC))
        .addAllSelection(selection)
        .setName(name)
        .putAllReviews(reviews)
        .setFinished(finished)
        .build()

    companion object {
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
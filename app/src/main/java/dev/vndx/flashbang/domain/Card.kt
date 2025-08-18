package dev.vndx.flashbang.domain

import dev.vndx.flashbang.domain.Tag
import uniffi.mobile.CardSource
import uniffi.mobile.FuzzyItem
import uniffi.mobile.Header
import java.time.LocalDate
import java.time.LocalDateTime

data class Card (
    val id: String,
    val name: String,
    val locations: List<Tag>,
    val question: String,
    val answer: String,
    val header: Header?,
    var scheduledFor: LocalDate,
) : CardSource, FuzzyItem {
    override fun header(): Header? = header

    override fun id(): String = id

    override fun name(): String = name

    override fun question(): String = question

    override fun answer(): String = answer

    override fun locations(): List<String> = locations.map { it.fullPath }

    override fun key(): String = name
}
package dev.vndx.flashbang.domain

import dev.vndx.flashbang.domain.Tag
import dev.vndx.flashbang.ui.LocalDateSerializer
import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.KSerializer
import kotlinx.serialization.Serializable
import kotlinx.serialization.Serializer
import kotlinx.serialization.serializerOrNull
import uniffi.mobile.CardSource
import uniffi.mobile.FuzzyItem
import java.time.LocalDate
import java.time.LocalDateTime
import kotlinx.serialization.builtins.nullable
import kotlinx.serialization.descriptors.PrimitiveKind
import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import java.time.format.DateTimeFormatter

@Serializable(with = HeaderSerializer::class)
class Header(value: String) {
    val content = value.intern()
}

@OptIn(ExperimentalSerializationApi::class)
object HeaderSerializer : KSerializer<Header> {

    override fun serialize(encoder: Encoder, value: Header) {
        encoder.encodeString(value.content)
    }

    override fun deserialize(decoder: Decoder): Header {
        return Header(decoder.decodeString())
    }

    override val descriptor: SerialDescriptor =
        PrimitiveSerialDescriptor("HEADER", PrimitiveKind.STRING)
}

@Serializable
data class Card(
    val id: String,
    val name: String,
    val locations: List<Tag>,
    val question: String,
    val answer: String,
    val header: Header?,
    @Serializable(with = LocalDateSerializer::class) var scheduledFor: LocalDate? = null,
) : CardSource, FuzzyItem, Item {
    override fun headerContent(): String? = header?.content

    override fun headerEq(other: CardSource?): Boolean = other?.headerContent() == header?.content

    override fun id(): String = id

    override fun name(): String = name

    override fun question(): String = question

    override fun answer(): String = answer

    override fun locations(): List<String> = locations.map { it.fullPath }

    override fun key(): String = name

    override fun data(): String = id

    override val itemName: String
        get() = name
    override val childItems: List<Item>
        get() = emptyList()
    override val leafItems: List<Item>
        get() = listOf(this)
    override val parentItems: List<Item>
        get() = locations
}
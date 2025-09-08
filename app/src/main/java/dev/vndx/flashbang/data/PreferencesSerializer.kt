package dev.vndx.flashbang.data

import androidx.datastore.core.CorruptionException
import androidx.datastore.core.Serializer
import com.google.protobuf.InvalidProtocolBufferException
import dev.vndx.flashbang.DateFormat
import dev.vndx.flashbang.Preferences
import dev.vndx.flashbang.Theme
import java.io.InputStream
import java.io.OutputStream
import java.time.format.DateTimeFormatter

fun DateFormat.dateTimeFormatter(): DateTimeFormatter = when (this) {
    DateFormat.DATE_FORMAT_SLASH_DD_MM_YY -> DateTimeFormatter.ofPattern("dd/MM/uu")
    DateFormat.DATE_FORMAT_SLASH_MM_DD_YY -> DateTimeFormatter.ofPattern("MM/dd/uu")
    DateFormat.DATE_FORMAT_DASH_YYYY_MM_DD -> DateTimeFormatter.ofPattern("uuuu-MM-dd")
    DateFormat.DATE_FORMAT_SPACE_DD_MMM_YYYY -> DateTimeFormatter.ofPattern("dd MMM uuuu")
    DateFormat.DATE_FORMAT_SPACE_MMM_DD_YYYY -> DateTimeFormatter.ofPattern("MMM dd, uuuu")
    else -> DateTimeFormatter.ofPattern("d/M/uu")
}

object PreferencesSerializer : Serializer<Preferences> {
    override val defaultValue: Preferences
        get() = Preferences.newBuilder()
            .setUseDynamicColors(true)
            .setTheme(Theme.THEME_SYSTEM)
            .setBranch("main")
            .setDateFormat(DateFormat.DATE_FORMAT_SLASH_D_M_YY)
            .setCardFontSize(14)
            .build()

    override suspend fun readFrom(input: InputStream): Preferences {
        try {
            return Preferences.parseFrom(input)
        } catch (exception: InvalidProtocolBufferException) {
            throw CorruptionException("Cannot read proto.", exception)
        }
    }

    override suspend fun writeTo(t: Preferences, output: OutputStream) = t.writeTo(output)
}
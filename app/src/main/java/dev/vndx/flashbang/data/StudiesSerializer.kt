package dev.vndx.flashbang.data

import androidx.datastore.core.CorruptionException
import androidx.datastore.core.Serializer
import com.google.protobuf.InvalidProtocolBufferException
import dev.vndx.flashbang.Studies
import uniffi.mobile.Rating
import dev.vndx.flashbang.Rating as ProtoRating
import java.io.InputStream
import java.io.OutputStream

fun Rating.toProto() = when (this) {
    Rating.AGAIN -> ProtoRating.RATING_AGAIN
    Rating.HARD -> ProtoRating.RATING_HARD
    Rating.GOOD -> ProtoRating.RATING_GOOD
    Rating.EASY -> ProtoRating.RATING_EASY
}

fun ProtoRating.toFFI() = when (this) {
    ProtoRating.RATING_HARD  -> Rating.HARD
    ProtoRating.RATING_GOOD  -> Rating.GOOD
    ProtoRating.RATING_EASY  -> Rating.EASY
    else -> Rating.AGAIN
}
val DefaultParameters = listOf(
    0.212f,
    1.2931f,
    2.3065f,
    8.2956f,
    6.4133f,
    0.8334f,
    3.0194f,
    0.001f,
    1.8722f,
    0.1666f,
    0.796f,
    1.4835f,
    0.0614f,
    0.2629f,
    1.6483f,
    0.6014f,
    1.8729f,
    0.5425f,
    0.0912f,
    0.0658f,
    0.1542f,
);

object StudiesSerializer : Serializer<Studies> {
    override val defaultValue: Studies
        get() = Studies.newBuilder().setIds(0).addAllFSRSParameters(DefaultParameters).build()

    override suspend fun readFrom(input: InputStream): Studies {
        try {
            return Studies.parseFrom(input)
        } catch (exception: InvalidProtocolBufferException) {
            throw CorruptionException("Cannot read proto.", exception)
        }
    }

    override suspend fun writeTo(t: Studies, output: OutputStream) = t.writeTo(output)
}

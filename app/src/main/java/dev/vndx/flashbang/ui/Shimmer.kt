package dev.vndx.flashbang.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shape
import com.valentinilk.shimmer.shimmer

@Composable
fun Modifier.shimmerable(
    color: Color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.4f),
    shape: Shape = RoundedCornerShape(Sizes.cornerRadiusMedium)
): Modifier {
    if (!LocalShimmerState.current.isLoading) return this

    return this
        .shimmer()
        .background(color = color, shape = shape)
        .drawWithContent {}
}

data class ShimmerState(val isLoading: Boolean)

val LocalShimmerState = compositionLocalOf { ShimmerState(isLoading = false) }

@Composable
fun ShimmerProvider(isLoading: Boolean = true, content: @Composable (Boolean) -> Unit) {
    CompositionLocalProvider(
        value = LocalShimmerState provides ShimmerState(isLoading = isLoading),
    ) {
        content(isLoading)
    }
}
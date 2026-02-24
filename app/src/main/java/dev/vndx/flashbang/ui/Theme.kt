package dev.vndx.flashbang.ui

import android.annotation.SuppressLint
import android.os.Build
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.ExperimentalTextApi
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontVariation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dev.vndx.flashbang.R
import androidx.compose.ui.graphics.Color

val primaryLight = Color(0xFF445E91)
val onPrimaryLight = Color(0xFFFFFFFF)
val primaryContainerLight = Color(0xFFD8E2FF)
val onPrimaryContainerLight = Color(0xFF2B4678)
val secondaryLight = Color(0xFF425E91)
val onSecondaryLight = Color(0xFFFFFFFF)
val secondaryContainerLight = Color(0xFFD7E2FF)
val onSecondaryContainerLight = Color(0xFF294677)
val tertiaryLight = Color(0xFF5B5891)
val onTertiaryLight = Color(0xFFFFFFFF)
val tertiaryContainerLight = Color(0xFFE3DFFF)
val onTertiaryContainerLight = Color(0xFF434078)
val errorLight = Color(0xFFBA1A1A)
val onErrorLight = Color(0xFFFFFFFF)
val errorContainerLight = Color(0xFFFFDAD6)
val onErrorContainerLight = Color(0xFF93000A)
val backgroundLight = Color(0xFFF9F9FF)
val onBackgroundLight = Color(0xFF1A1B20)
val surfaceLight = Color(0xFFF9F9FF)
val onSurfaceLight = Color(0xFF1A1B20)
val surfaceVariantLight = Color(0xFFE1E2EC)
val onSurfaceVariantLight = Color(0xFF44474F)
val outlineLight = Color(0xFF74777F)
val outlineVariantLight = Color(0xFFC4C6D0)
val scrimLight = Color(0xFF000000)
val inverseSurfaceLight = Color(0xFF2F3036)
val inverseOnSurfaceLight = Color(0xFFF0F0F7)
val inversePrimaryLight = Color(0xFFADC6FF)
val surfaceDimLight = Color(0xFFD9D9E0)
val surfaceBrightLight = Color(0xFFF9F9FF)
val surfaceContainerLowestLight = Color(0xFFFFFFFF)
val surfaceContainerLowLight = Color(0xFFF3F3FA)
val surfaceContainerLight = Color(0xFFEDEDF4)
val surfaceContainerHighLight = Color(0xFFE8E7EE)
val surfaceContainerHighestLight = Color(0xFFE2E2E9)


val primaryDark = Color(0xFFADC6FF)
val onPrimaryDark = Color(0xFF102F60)
val primaryContainerDark = Color(0xFF2B4678)
val onPrimaryContainerDark = Color(0xFFD8E2FF)
val secondaryDark = Color(0xFFACC7FF)
val onSecondaryDark = Color(0xFF0E2F60)
val secondaryContainerDark = Color(0xFF294677)
val onSecondaryContainerDark = Color(0xFFD7E2FF)
val tertiaryDark = Color(0xFFC4C0FF)
val onTertiaryDark = Color(0xFF2D2960)
val tertiaryContainerDark = Color(0xFF434078)
val onTertiaryContainerDark = Color(0xFFE3DFFF)
val errorDark = Color(0xFFFFB4AB)
val onErrorDark = Color(0xFF690005)
val errorContainerDark = Color(0xFF93000A)
val onErrorContainerDark = Color(0xFFFFDAD6)
val backgroundDark = Color(0xFF111318)
val onBackgroundDark = Color(0xFFE2E2E9)
val surfaceDark = Color(0xFF111318)
val onSurfaceDark = Color(0xFFE2E2E9)
val surfaceVariantDark = Color(0xFF44474F)
val onSurfaceVariantDark = Color(0xFFC4C6D0)
val outlineDark = Color(0xFF8E9099)
val outlineVariantDark = Color(0xFF44474F)
val scrimDark = Color(0xFF000000)
val inverseSurfaceDark = Color(0xFFE2E2E9)
val inverseOnSurfaceDark = Color(0xFF2F3036)
val inversePrimaryDark = Color(0xFF445E91)
val surfaceDimDark = Color(0xFF111318)
val surfaceBrightDark = Color(0xFF37393E)
val surfaceContainerLowestDark = Color(0xFF0C0E13)
val surfaceContainerLowDark = Color(0xFF1A1B20)
val surfaceContainerDark = Color(0xFF1E1F25)
val surfaceContainerHighDark = Color(0xFF282A2F)
val surfaceContainerHighestDark = Color(0xFF33353A)

private val lightScheme = lightColorScheme(
    primary = primaryLight,
    onPrimary = onPrimaryLight,
    primaryContainer = primaryContainerLight,
    onPrimaryContainer = onPrimaryContainerLight,
    secondary = secondaryLight,
    onSecondary = onSecondaryLight,
    secondaryContainer = secondaryContainerLight,
    onSecondaryContainer = onSecondaryContainerLight,
    tertiary = tertiaryLight,
    onTertiary = onTertiaryLight,
    tertiaryContainer = tertiaryContainerLight,
    onTertiaryContainer = onTertiaryContainerLight,
    error = errorLight,
    onError = onErrorLight,
    errorContainer = errorContainerLight,
    onErrorContainer = onErrorContainerLight,
    background = backgroundLight,
    onBackground = onBackgroundLight,
    surface = surfaceLight,
    onSurface = onSurfaceLight,
    surfaceVariant = surfaceVariantLight,
    onSurfaceVariant = onSurfaceVariantLight,
    outline = outlineLight,
    outlineVariant = outlineVariantLight,
    scrim = scrimLight,
    inverseSurface = inverseSurfaceLight,
    inverseOnSurface = inverseOnSurfaceLight,
    inversePrimary = inversePrimaryLight,
    surfaceDim = surfaceDimLight,
    surfaceBright = surfaceBrightLight,
    surfaceContainerLowest = surfaceContainerLowestLight,
    surfaceContainerLow = surfaceContainerLowLight,
    surfaceContainer = surfaceContainerLight,
    surfaceContainerHigh = surfaceContainerHighLight,
    surfaceContainerHighest = surfaceContainerHighestLight,
)

private val darkScheme = darkColorScheme(
    primary = primaryDark,
    onPrimary = onPrimaryDark,
    primaryContainer = primaryContainerDark,
    onPrimaryContainer = onPrimaryContainerDark,
    secondary = secondaryDark,
    onSecondary = onSecondaryDark,
    secondaryContainer = secondaryContainerDark,
    onSecondaryContainer = onSecondaryContainerDark,
    tertiary = tertiaryDark,
    onTertiary = onTertiaryDark,
    tertiaryContainer = tertiaryContainerDark,
    onTertiaryContainer = onTertiaryContainerDark,
    error = errorDark,
    onError = onErrorDark,
    errorContainer = errorContainerDark,
    onErrorContainer = onErrorContainerDark,
    background = backgroundDark,
    onBackground = onBackgroundDark,
    surface = surfaceDark,
    onSurface = onSurfaceDark,
    surfaceVariant = surfaceVariantDark,
    onSurfaceVariant = onSurfaceVariantDark,
    outline = outlineDark,
    outlineVariant = outlineVariantDark,
    scrim = scrimDark,
    inverseSurface = inverseSurfaceDark,
    inverseOnSurface = inverseOnSurfaceDark,
    inversePrimary = inversePrimaryDark,
    surfaceDim = surfaceDimDark,
    surfaceBright = surfaceBrightDark,
    surfaceContainerLowest = surfaceContainerLowestDark,
    surfaceContainerLow = surfaceContainerLowDark,
    surfaceContainer = surfaceContainerDark,
    surfaceContainerHigh = surfaceContainerHighDark,
    surfaceContainerHighest = surfaceContainerHighestDark,
)

@OptIn(ExperimentalTextApi::class)
private fun lexend(weight: Int): FontFamily {
    return FontFamily(
        Font(
            R.font.lexend_variable_wght, variationSettings = FontVariation.Settings(
                FontVariation.weight(weight)
            )
        )
    )
}

val lexendRegular = lexend(400)
val lexendMedium = lexend(500)
val lexendBold = lexend(700)

val typography = Typography(
    displayLarge = TextStyle(fontFamily = lexendMedium, fontSize = 57.sp, lineHeight = 64.sp),
    displayMedium = TextStyle(fontFamily = lexendMedium, fontSize = 45.sp, lineHeight = 52.sp),
    displaySmall = TextStyle(fontFamily = lexendMedium, fontSize = 36.sp, lineHeight = 44.sp),

    headlineLarge = TextStyle(fontFamily = lexendMedium, fontSize = 32.sp, lineHeight = 40.sp),
    headlineMedium = TextStyle(fontFamily = lexendMedium, fontSize = 28.sp, lineHeight = 36.sp),
    headlineSmall = TextStyle(fontFamily = lexendMedium, fontSize = 20.sp, lineHeight = 26.sp),

    titleLarge = TextStyle(fontFamily = lexendBold, fontSize = 22.sp, lineHeight = 28.sp),
    titleMedium = TextStyle(fontFamily = lexendBold, fontSize = 16.sp, lineHeight = 24.sp),
    titleSmall = TextStyle(fontFamily = lexendBold, fontSize = 14.sp, lineHeight = 20.sp),

    bodyLarge = TextStyle(fontFamily = lexendRegular, fontSize = 16.sp, lineHeight = 24.sp),
    bodyMedium = TextStyle(fontFamily = lexendRegular, fontSize = 14.sp, lineHeight = 20.sp),
    bodySmall = TextStyle(fontFamily = lexendRegular, fontSize = 12.sp, lineHeight = 16.sp),

    labelLarge = TextStyle(fontFamily = lexendMedium, fontSize = 18.sp, lineHeight = 20.sp),
    labelMedium = TextStyle(fontFamily = lexendMedium, fontSize = 16.sp, lineHeight = 16.sp),
    labelSmall = TextStyle(fontFamily = lexendMedium, fontSize = 14.sp, lineHeight = 16.sp),
)

data object Sizes {
    val cornerRadiusMedium = 7.dp
    val cornerRadiusLarge = 9.dp
    val cornerRadiusHuge = 15.dp
    val spacingMinuscule = 4.dp
    val spacingTiny = 5.dp
    val spacingSmall = 10.dp
    val spacingMedium = 15.dp
    val spacingLarge = 20.dp
    val spacingHuge = 30.dp
    val borderWidth = 2.dp
}

// Lint is suppressed because I might change the minimum sdk later on
@SuppressLint("ObsoleteSdkInt")
@Composable
fun FlashbangTheme(useDarkTheme: Boolean = false, useDynamicColors: Boolean = true, content: @Composable() () -> Unit) {
    val colors = when {
        Build.VERSION.SDK_INT >= Build.VERSION_CODES.S && useDynamicColors -> {
            if (useDarkTheme) dynamicDarkColorScheme(LocalContext.current)
            else dynamicLightColorScheme(LocalContext.current)
        }

        useDarkTheme -> darkScheme
        else -> lightScheme
    }

    MaterialTheme(
        colorScheme = colors,
        content = content,
        typography = typography,
    )
}
package dev.vndx.flashbang.ui

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.pluralStringResource
import androidx.compose.ui.res.stringResource
import java.time.LocalDate
import java.time.format.DateTimeFormatter
import java.time.format.FormatStyle
import java.time.temporal.ChronoUnit
import dev.vndx.flashbang.R

@Composable
fun formatRelativeDate(
    date: LocalDate,
    relative: Boolean = true,
    format: DateTimeFormatter = DateTimeFormatter.ofLocalizedDate(
        FormatStyle.SHORT
    )
): String {
    if (!relative) {
        return format.format(date)
    }

    val now = LocalDate.now()
    val difference = ChronoUnit.DAYS.between(date, now)
    return when {
        difference <= -14 -> format.format(date)
        difference <= -7 -> stringResource(R.string.next_week)
        difference < 0 -> pluralStringResource(R.plurals.days_ahead, (-difference).toInt(), -difference)
        difference < 1 -> stringResource(R.string.today)
        difference < 7 -> pluralStringResource(R.plurals.days_before, difference.toInt(), difference)
        difference < 14 -> stringResource(R.string.last_week)
        else -> format.format(date)
    }
}
package dev.vndx.flashbang.ui.screens

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import dev.vndx.flashbang.R
import dev.vndx.flashbang.Rating
import dev.vndx.flashbang.domain.Study
import dev.vndx.flashbang.ui.ChartBar
import dev.vndx.flashbang.ui.SimpleBarChart
import dev.vndx.flashbang.ui.Sizes
import kotlinx.serialization.Serializable
import java.time.Duration
import java.time.LocalDateTime

@Serializable
class StudyFinishedScreen(val study: Study) : Screen {

    override fun tab(): Tab = Tab.Study

    @Composable
    override fun Compose(onNavigate: (Screen) -> Unit, onBack: (Int?) -> Unit) {
        // Since we are replacing the ReviewScreen, onBack(1) should take us to Home (or whatever was before Review).

        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(Sizes.spacingMedium),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center
        ) {
            Text(
                text = stringResource(R.string.study_finished),
                style = MaterialTheme.typography.headlineLarge,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(bottom = Sizes.spacingLarge)
            )

            // Duration
            val now = remember { LocalDateTime.now() }
            val duration = Duration.between(study.timestamp, now)
            if (duration.toDays() < 1) {
                val durationString = formatDuration(duration)
                Text(
                    text = stringResource(R.string.study_duration, durationString),
                    style = MaterialTheme.typography.titleMedium,
                    modifier = Modifier.padding(bottom = Sizes.spacingSmall)
                )
            }

            // Cards Reviewed
            val reviews = study.reviews
            val totalReviewed = reviews.size
            Text(
                text = stringResource(R.string.study_cards_reviewed, totalReviewed),
                style = MaterialTheme.typography.titleMedium,
                modifier = Modifier.padding(bottom = Sizes.spacingLarge)
            )

            Spacer(modifier = Modifier.height(Sizes.spacingLarge))

            // Bar Chart
            if (totalReviewed > 0) {
                val againCount = reviews.values.count { it == Rating.RATING_AGAIN }
                val hardCount = reviews.values.count { it == Rating.RATING_HARD }
                val goodCount = reviews.values.count { it == Rating.RATING_GOOD }
                val easyCount = reviews.values.count { it == Rating.RATING_EASY }

                val data = listOf(
                    ChartBar(stringResource(R.string.rating_again), againCount.toFloat(), MaterialTheme.colorScheme.inverseSurface),
                    ChartBar(stringResource(R.string.rating_hard), hardCount.toFloat(), MaterialTheme.colorScheme.error),
                    ChartBar(stringResource(R.string.rating_good), goodCount.toFloat(), MaterialTheme.colorScheme.tertiary),
                    ChartBar(stringResource(R.string.rating_easy), easyCount.toFloat(), MaterialTheme.colorScheme.primary)
                )

                SimpleBarChart(
                    data = data,
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(200.dp)
                        .padding(horizontal = Sizes.spacingMedium)
                )
            } else {
                Text(stringResource(R.string.study_no_cards))
            }

            Spacer(modifier = Modifier.height(Sizes.spacingHuge))

            Button(
                onClick = { onBack(1) },
                modifier = Modifier.fillMaxWidth()
            ) {
                Text(stringResource(R.string.back_to_home))
            }
        }
    }

    private fun formatDuration(duration: Duration): String {
        val hours = duration.toHours()
        val minutes = duration.toMinutes() % 60
        val seconds = duration.seconds % 60
        return if (hours > 0) {
            String.format("%d:%02d:%02d", hours, minutes, seconds)
        } else {
            String.format("%02d:%02d", minutes, seconds)
        }
    }
}

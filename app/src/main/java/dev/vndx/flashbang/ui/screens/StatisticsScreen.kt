package dev.vndx.flashbang.ui.screens

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Card
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import dev.vndx.flashbang.ui.ChartBar
import dev.vndx.flashbang.ui.SimpleBarChart
import dev.vndx.flashbang.ui.StackedBar
import dev.vndx.flashbang.ui.StackedBarChart
import dev.vndx.flashbang.ui.StatisticsState
import dev.vndx.flashbang.ui.StatisticsViewModel
import java.time.format.DateTimeFormatter
import kotlinx.serialization.Serializable

@Serializable
class StatisticsScreen : Screen {

    override fun tab() = Tab.Statistics
    override fun showTabs() = true
    override fun isHomeScreen() = false

    @Composable
    override fun Compose(onNavigate: (Screen) -> Unit, onBack: (Int?) -> Unit) {
        val viewModel = viewModel<StatisticsViewModel>()
        val state by viewModel.statisticsState.collectAsState()

        when (val s = state) {
            is StatisticsState.Loading -> {
                Column(
                    modifier = Modifier.fillMaxSize(),
                    verticalArrangement = Arrangement.Center,
                    horizontalAlignment = Alignment.CenterHorizontally
                ) {
                    CircularProgressIndicator()
                }
            }
            is StatisticsState.Success -> {
                StatisticsContent(s.data)
            }
        }
    }
}

@Composable
fun StatisticsContent(data: dev.vndx.flashbang.ui.StatisticsData) {
    val scrollState = rememberScrollState()
    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(scrollState)
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(24.dp)
    ) {
        // Today Stats
        Text("Today", style = MaterialTheme.typography.headlineSmall)
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            StatItem("Reviews", data.today.totalReviews.toString())
            StatItem("Again", "${data.today.againCount} (${(data.today.passRate * 100).toInt()}%)")
            StatItem("Learned", data.today.learned.toString())
            StatItem("Reviewed", data.today.reviewed.toString())
            StatItem("Relearned", data.today.relearned.toString())
        }

        // Forecast
        Text("Forecast", style = MaterialTheme.typography.headlineSmall)
        val forecastData = remember(data.futureDue) {
            val formatter = DateTimeFormatter.ofPattern("MM-dd")
            data.futureDue.map { (date, count) ->
                ChartBar(
                    label = date.format(formatter),
                    value = count.toFloat(),
                    color = Color(0xFF2196F3) // Blue
                )
            }
        }
        if (forecastData.isNotEmpty()) {
            SimpleBarChart(
                data = forecastData,
                modifier = Modifier
                    .fillMaxWidth()
                    .height(200.dp)
            )
        } else {
            Text("No future reviews scheduled.")
        }

        // Review History
        Text("Review History", style = MaterialTheme.typography.headlineSmall)
        val historyData = remember(data.reviewHistory) {
            val formatter = DateTimeFormatter.ofPattern("MM-dd")
            data.reviewHistory.map { (date, stats) ->
                StackedBar(
                    label = date.format(formatter),
                    values = listOf(stats.again.toFloat(), stats.hard.toFloat(), stats.good.toFloat(), stats.easy.toFloat()),
                    colors = listOf(
                        Color(0xFFFF5252), // Red (Again)
                        Color(0xFFFFAB40), // Orange (Hard)
                        Color(0xFF69F0AE), // Green (Good)
                        Color(0xFF40C4FF)  // Blue (Easy)
                    )
                )
            }
        }
        if (historyData.isNotEmpty()) {
            StackedBarChart(
                data = historyData,
                modifier = Modifier
                    .fillMaxWidth()
                    .height(200.dp)
            )
        } else {
            Text("No review history.")
        }

        // Card Counts
        Text("Card Counts", style = MaterialTheme.typography.headlineSmall)
        val countsData = remember(data.cardCounts) {
            listOf(
                ChartBar("Young", data.cardCounts.young.toFloat(), Color(0xFF69F0AE)),
                ChartBar("Mature", data.cardCounts.mature.toFloat(), Color(0xFF40C4FF))
            )
        }
        SimpleBarChart(
            data = countsData,
            modifier = Modifier
                .fillMaxWidth()
                .height(150.dp),
            barSpacing = 32f
        )
        Row(horizontalArrangement = Arrangement.spacedBy(16.dp)) {
            Text("Young: ${data.cardCounts.young}", style = MaterialTheme.typography.bodyMedium, color = Color(0xFF69F0AE))
            Text("Mature: ${data.cardCounts.mature}", style = MaterialTheme.typography.bodyMedium, color = Color(0xFF40C4FF))
            Text("Total: ${data.cardCounts.total}", style = MaterialTheme.typography.bodyMedium)
        }

        // Stability Distribution
        Text("Card Stability (Days)", style = MaterialTheme.typography.headlineSmall)
        val stabilityData = remember(data.stabilityDistribution) {
            data.stabilityDistribution.map { (days, count) ->
                ChartBar(days.toString(), count.toFloat(), Color(0xFFAB47BC)) // Purple
            }
        }
        if (stabilityData.isNotEmpty()) {
            SimpleBarChart(
                data = stabilityData,
                modifier = Modifier
                    .fillMaxWidth()
                    .height(150.dp)
            )
        }

        // Difficulty Distribution
        Text("Card Difficulty", style = MaterialTheme.typography.headlineSmall)
        val difficultyData = remember(data.difficultyDistribution) {
            data.difficultyDistribution.map { (diffX10, count) ->
                ChartBar((diffX10 / 10f).toString(), count.toFloat(), Color(0xFFFF7043)) // Deep Orange
            }
        }
        if (difficultyData.isNotEmpty()) {
            SimpleBarChart(
                data = difficultyData,
                modifier = Modifier
                    .fillMaxWidth()
                    .height(150.dp)
            )
        }

        Spacer(modifier = Modifier.height(32.dp))
    }
}

@Composable
fun StatItem(label: String, value: String) {
    Column(horizontalAlignment = Alignment.CenterHorizontally) {
        Text(text = value, style = MaterialTheme.typography.titleMedium)
        Text(text = label, style = MaterialTheme.typography.bodySmall)
    }
}

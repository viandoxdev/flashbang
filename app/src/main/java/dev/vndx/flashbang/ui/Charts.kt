package dev.vndx.flashbang.ui

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import com.patrykandpatrick.vico.compose.axis.horizontal.rememberBottomAxis
import com.patrykandpatrick.vico.compose.axis.vertical.rememberStartAxis
import com.patrykandpatrick.vico.compose.chart.Chart
import com.patrykandpatrick.vico.compose.chart.column.columnChart
import com.patrykandpatrick.vico.compose.m3.style.m3ChartStyle
import com.patrykandpatrick.vico.compose.style.ProvideChartStyle
import com.patrykandpatrick.vico.core.axis.AxisPosition
import com.patrykandpatrick.vico.core.axis.formatter.AxisValueFormatter
import com.patrykandpatrick.vico.core.chart.column.ColumnChart
import com.patrykandpatrick.vico.core.component.shape.LineComponent
import com.patrykandpatrick.vico.core.entry.ChartEntryModelProducer
import com.patrykandpatrick.vico.core.entry.FloatEntry
import com.patrykandpatrick.vico.core.entry.entryModelOf

data class ChartBar(
    val label: String,
    val value: Float,
    val color: Color
)

data class StackedBar(
    val label: String,
    val values: List<Float>, // Bottom to top
    val colors: List<Color>
)

@Composable
fun SimpleBarChart(
    data: List<ChartBar>,
    modifier: Modifier = Modifier
) {
    if (data.isEmpty()) return

    val entries = data.mapIndexed { index, bar -> FloatEntry(index.toFloat(), bar.value) }
    val chartEntryModel = entryModelOf(entries)

    val horizontalAxisValueFormatter = AxisValueFormatter<AxisPosition.Horizontal.Bottom> { value, _ ->
        data.getOrNull(value.toInt())?.label ?: ""
    }

    ProvideChartStyle(m3ChartStyle()) {
        Chart(
            chart = columnChart(
                columns = listOf(
                    LineComponent(
                        color = (data.firstOrNull()?.color ?: MaterialTheme.colorScheme.primary).toArgb(),
                        thicknessDp = 16f,
                    )
                )
            ),
            model = chartEntryModel,
            startAxis = rememberStartAxis(),
            bottomAxis = rememberBottomAxis(
                valueFormatter = horizontalAxisValueFormatter
            ),
            modifier = modifier
        )
    }
}

@Composable
fun StackedBarChart(
    data: List<StackedBar>,
    modifier: Modifier = Modifier
) {
    if (data.isEmpty()) return

    // Vico expects List<List<Entry>> where outer list is series (stack layer), inner is time points.
    val seriesCount = data.firstOrNull()?.values?.size ?: 0
    if (seriesCount == 0) return

    val chartEntries = (0 until seriesCount).map { seriesIndex ->
        data.mapIndexed { xIndex, bar ->
            FloatEntry(xIndex.toFloat(), bar.values.getOrElse(seriesIndex) { 0f })
        }
    }

    val chartEntryModel = entryModelOf(chartEntries)

    val horizontalAxisValueFormatter = AxisValueFormatter<AxisPosition.Horizontal.Bottom> { value, _ ->
        data.getOrNull(value.toInt())?.label ?: ""
    }

    // Use colors from the first bar to define series colors
    val firstBarColors = data.first().colors
    val columns = firstBarColors.map { color ->
        LineComponent(
            color = color.toArgb(),
            thicknessDp = 16f
        )
    }

    ProvideChartStyle(m3ChartStyle()) {
        Chart(
            chart = columnChart(
                columns = columns,
                mergeMode = ColumnChart.MergeMode.Stack
            ),
            model = chartEntryModel,
            startAxis = rememberStartAxis(),
            bottomAxis = rememberBottomAxis(
                valueFormatter = horizontalAxisValueFormatter
            ),
            modifier = modifier
        )
    }
}

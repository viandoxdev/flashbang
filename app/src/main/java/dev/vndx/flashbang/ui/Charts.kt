package dev.vndx.flashbang.ui

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.unit.dp
import com.patrykandpatrick.vico.compose.cartesian.CartesianChartHost
import com.patrykandpatrick.vico.compose.cartesian.axis.rememberBottomAxis
import com.patrykandpatrick.vico.compose.cartesian.axis.rememberStartAxis
import com.patrykandpatrick.vico.compose.cartesian.layer.rememberColumnCartesianLayer
import com.patrykandpatrick.vico.compose.cartesian.rememberCartesianChart
import com.patrykandpatrick.vico.compose.common.component.rememberLineComponent
import com.patrykandpatrick.vico.core.cartesian.axis.HorizontalAxis
import com.patrykandpatrick.vico.core.cartesian.axis.VerticalAxis
import com.patrykandpatrick.vico.core.cartesian.data.CartesianChartModelProducer
import com.patrykandpatrick.vico.core.cartesian.data.columnSeries
import com.patrykandpatrick.vico.core.cartesian.layer.ColumnCartesianLayer
import com.patrykandpatrick.vico.core.common.data.ExtraStore

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

val LabelKey = ExtraStore.Key<List<String>>()

@Composable
fun SimpleBarChart(
    data: List<ChartBar>,
    modifier: Modifier = Modifier
) {
    if (data.isEmpty()) return

    val modelProducer = remember { CartesianChartModelProducer() }

    LaunchedEffect(data) {
        modelProducer.runTransaction {
            columnSeries {
                series(data.map { it.value })
            }
            extras {
                it[LabelKey] = data.map { it.label }
            }
        }
    }

    CartesianChartHost(
        chart = rememberCartesianChart(
            rememberColumnCartesianLayer(
                columnProvider = ColumnCartesianLayer.ColumnProvider.series(
                    data.map {
                        rememberLineComponent(
                            color = it.color,
                            thickness = 16.dp
                        )
                    }
                )
            ),
            startAxis = VerticalAxis.rememberStart(),
            bottomAxis = HorizontalAxis.rememberBottom(
                valueFormatter = { value, chartValues, _ ->
                    val labels = chartValues.model.extraStore.getOrNull(LabelKey)
                    labels?.getOrNull(value.toInt()) ?: ""
                }
            )
        ),
        modelProducer = modelProducer,
        modifier = modifier
    )
}

@Composable
fun StackedBarChart(
    data: List<StackedBar>,
    modifier: Modifier = Modifier
) {
    if (data.isEmpty()) return

    val modelProducer = remember { CartesianChartModelProducer() }

    // Transpose data for series based model
    val seriesCount = data.first().values.size
    val transposedData = (0 until seriesCount).map { seriesIndex ->
        data.map { it.values.getOrElse(seriesIndex) { 0f } }
    }

    LaunchedEffect(data) {
        modelProducer.runTransaction {
            columnSeries {
                transposedData.forEach { series(it) }
            }
            extras {
                it[LabelKey] = data.map { it.label }
            }
        }
    }

    val colors = data.first().colors
    val columns = colors.map { color ->
        rememberLineComponent(
            color = color,
            thickness = 16.dp
        )
    }

    CartesianChartHost(
        chart = rememberCartesianChart(
            rememberColumnCartesianLayer(
                columnProvider = ColumnCartesianLayer.ColumnProvider.series(columns),
                mergeMode = { ColumnCartesianLayer.MergeMode.Stacked }
            ),
            startAxis = VerticalAxis.rememberStart(),
            bottomAxis = HorizontalAxis.rememberBottom(
                valueFormatter = { value, chartValues, _ ->
                    val labels = chartValues.model.extraStore.getOrNull(LabelKey)
                    labels?.getOrNull(value.toInt()) ?: ""
                }
            )
        ),
        modelProducer = modelProducer,
        modifier = modifier
    )
}

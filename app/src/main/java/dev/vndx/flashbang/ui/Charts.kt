package dev.vndx.flashbang.ui

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.unit.dp
import java.time.LocalDate
import java.time.format.DateTimeFormatter
import kotlin.math.roundToInt

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
    modifier: Modifier = Modifier,
    barSpacing: Float = 2f
) {
    if (data.isEmpty()) return

    val maxValue = remember(data) { data.maxOfOrNull { it.value } ?: 0f }
    val textColor = MaterialTheme.colorScheme.onSurface

    Box(modifier = modifier) {
        Canvas(modifier = Modifier.fillMaxSize()) {
            val chartHeight = size.height
            val chartWidth = size.width
            val barCount = data.size
            val barWidth = (chartWidth - (barSpacing * (barCount - 1))) / barCount

            data.forEachIndexed { index, bar ->
                val barHeight = if (maxValue > 0) (bar.value / maxValue) * chartHeight else 0f
                val x = index * (barWidth + barSpacing)
                val y = chartHeight - barHeight

                drawRect(
                    color = bar.color,
                    topLeft = Offset(x, y),
                    size = Size(barWidth, barHeight)
                )
            }

            // Draw axis line
            drawLine(
                color = textColor.copy(alpha = 0.5f),
                start = Offset(0f, chartHeight),
                end = Offset(chartWidth, chartHeight),
                strokeWidth = 2f
            )
        }

        // Max value label
        Text(
            text = maxValue.roundToInt().toString(),
            style = MaterialTheme.typography.labelSmall,
            modifier = Modifier.align(Alignment.TopStart)
        )
    }
}

@Composable
fun StackedBarChart(
    data: List<StackedBar>,
    modifier: Modifier = Modifier,
    barSpacing: Float = 1f
) {
    if (data.isEmpty()) return

    val maxValue = remember(data) {
        data.maxOfOrNull { it.values.sum() } ?: 0f
    }
    val textColor = MaterialTheme.colorScheme.onSurface

    Box(modifier = modifier) {
        Canvas(modifier = Modifier.fillMaxSize()) {
            val chartHeight = size.height
            val chartWidth = size.width
            val barCount = data.size
            // Use minimal spacing if many bars
            val effectiveSpacing = if (barCount > 50) 0f else barSpacing
            val barWidth = (chartWidth - (effectiveSpacing * (barCount - 1))) / barCount.coerceAtLeast(1)

            data.forEachIndexed { index, bar ->
                val x = index * (barWidth + effectiveSpacing)
                var currentY = chartHeight

                bar.values.forEachIndexed { vIndex, value ->
                    val segmentHeight = if (maxValue > 0) (value / maxValue) * chartHeight else 0f
                    val y = currentY - segmentHeight

                    drawRect(
                        color = bar.colors.getOrElse(vIndex) { Color.Gray },
                        topLeft = Offset(x, y),
                        size = Size(barWidth, segmentHeight)
                    )

                    currentY -= segmentHeight
                }
            }

            // Axis
            drawLine(
                color = textColor.copy(alpha = 0.5f),
                start = Offset(0f, chartHeight),
                end = Offset(chartWidth, chartHeight),
                strokeWidth = 2f
            )
        }

        // Labels
        Text(
            text = maxValue.roundToInt().toString(),
            style = MaterialTheme.typography.labelSmall,
            modifier = Modifier.align(Alignment.TopStart)
        )

        // Start Date Label
        if (data.isNotEmpty()) {
            Text(
                text = data.first().label,
                style = MaterialTheme.typography.labelSmall,
                modifier = Modifier.align(Alignment.BottomStart).padding(top = 4.dp)
            )
        }

        // End Date Label
        if (data.size > 1) {
            Text(
                text = data.last().label,
                style = MaterialTheme.typography.labelSmall,
                modifier = Modifier.align(Alignment.BottomEnd).padding(top = 4.dp)
            )
        }
    }
}

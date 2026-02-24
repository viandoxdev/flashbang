package dev.vndx.flashbang.ui

import androidx.compose.foundation.clickable
import dev.vndx.flashbang.R
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.CenterAlignedTopAppBar
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.unit.dp

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun TopBar(
    title: String,
    onBack: (() -> Unit)? = null,
    actions: (@Composable RowScope.() -> Unit)? = null
) {
    val title = @Composable {
        Text(
            title, style = MaterialTheme.typography.titleLarge,
        )
    }
    val colors = TopAppBarDefaults.centerAlignedTopAppBarColors(
        containerColor = MaterialTheme.colorScheme.background,
        titleContentColor = MaterialTheme.colorScheme.onBackground
    )
    val navigationIcon = onBack?.let {
        @Composable {
            IconButton(onClick = it) {
                Icon(
                    painter = painterResource(R.drawable.outline_arrow_back_32),
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.onBackground
                )
            }
        }
    }

    // Disgusting but ehh
    if (navigationIcon != null) {
        if (actions != null) {
            CenterAlignedTopAppBar(
                title,
                navigationIcon = navigationIcon,
                actions = actions,
                colors = colors
            )
        } else {
            CenterAlignedTopAppBar(title, navigationIcon = navigationIcon, colors = colors)
        }
    } else {
        if (actions != null) {
            CenterAlignedTopAppBar(title, actions = actions, colors = colors)
        } else {
            CenterAlignedTopAppBar(title, colors = colors)
        }
    }
}

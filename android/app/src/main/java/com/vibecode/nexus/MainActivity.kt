package com.vibecode.nexus

import android.Manifest
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Checklist
import androidx.compose.material.icons.filled.Mic
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material.icons.automirrored.filled.TrendingUp
import androidx.compose.material3.Icon
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.core.content.ContextCompat
import androidx.core.content.PermissionChecker
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import com.vibecode.nexus.data.ConnectionSettings
import com.vibecode.nexus.data.NexusApiClient
import com.vibecode.nexus.speech.SpeechRecognizerManager
import com.vibecode.nexus.ui.screen.BrainDumpScreen
import com.vibecode.nexus.ui.screen.ProjectsScreen
import com.vibecode.nexus.ui.screen.SettingsScreen
import com.vibecode.nexus.ui.screen.TasksScreen
import com.vibecode.nexus.ui.theme.NexusTheme
import kotlinx.coroutines.delay

data class BottomNavItem(
    val route: String,
    val label: String,
    val icon: ImageVector
)

class MainActivity : ComponentActivity() {

    private val bottomNavItems = listOf(
        BottomNavItem("braindump", "BrainDump", Icons.Default.Mic),
        BottomNavItem("tasks", "Tasks", Icons.Default.Checklist),
        BottomNavItem("projects", "Projekte", Icons.AutoMirrored.Filled.TrendingUp),
        BottomNavItem("settings", "Settings", Icons.Default.Settings),
    )

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            NexusTheme {
                val navController = rememberNavController()
                val connectionSettings = remember { ConnectionSettings(this) }
                val apiClient = remember { NexusApiClient(connectionSettings) }
                val speechManager = remember { SpeechRecognizerManager(this) }

                DisposableEffect(Unit) {
                    speechManager.initialize()
                    onDispose {
                        speechManager.destroy()
                        apiClient.close()
                    }
                }

                var hasAudioPermission by remember {
                    mutableStateOf(
                        ContextCompat.checkSelfPermission(
                            this, Manifest.permission.RECORD_AUDIO
                        ) == PermissionChecker.PERMISSION_GRANTED
                    )
                }

                val audioPermissionLauncher = rememberLauncherForActivityResult(
                    ActivityResultContracts.RequestPermission()
                ) { granted ->
                    hasAudioPermission = granted
                }

                // Periodic health check
                var isConnected by remember { mutableStateOf<Boolean?>(null) }
                var isPaired by remember { mutableStateOf(connectionSettings.isPaired) }

                LaunchedEffect(isPaired) {
                    while (true) {
                        if (isPaired) {
                            isConnected = apiClient.checkHealth()
                        }
                        delay(15_000)
                    }
                }

                // Refresh paired state when navigating
                val navBackStackEntry by navController.currentBackStackEntryAsState()
                LaunchedEffect(navBackStackEntry) {
                    isPaired = connectionSettings.isPaired
                }

                val currentRoute = navBackStackEntry?.destination?.route

                Scaffold(
                    bottomBar = {
                        NavigationBar {
                            bottomNavItems.forEach { item ->
                                NavigationBarItem(
                                    selected = currentRoute == item.route,
                                    onClick = {
                                        if (currentRoute != item.route) {
                                            navController.navigate(item.route) {
                                                popUpTo(navController.graph.startDestinationId) {
                                                    saveState = true
                                                }
                                                launchSingleTop = true
                                                restoreState = true
                                            }
                                        }
                                    },
                                    icon = { Icon(item.icon, contentDescription = item.label) },
                                    label = { Text(item.label) }
                                )
                            }
                        }
                    }
                ) { innerPadding ->
                    NavHost(
                        navController = navController,
                        startDestination = "braindump"
                    ) {
                        composable("braindump") {
                            BrainDumpScreen(
                                speechManager = speechManager,
                                apiClient = apiClient,
                                isPaired = isPaired,
                                isConnected = isConnected,
                                hasPermission = hasAudioPermission,
                                onRequestPermission = {
                                    audioPermissionLauncher.launch(Manifest.permission.RECORD_AUDIO)
                                }
                            )
                        }
                        composable("tasks") {
                            TasksScreen(
                                apiClient = apiClient,
                                isPaired = isPaired
                            )
                        }
                        composable("projects") {
                            ProjectsScreen(
                                apiClient = apiClient,
                                isPaired = isPaired
                            )
                        }
                        composable("settings") {
                            SettingsScreen(
                                connectionSettings = connectionSettings,
                                apiClient = apiClient,
                                onNavigateBack = {
                                    navController.popBackStack()
                                }
                            )
                        }
                    }
                }
            }
        }
    }
}

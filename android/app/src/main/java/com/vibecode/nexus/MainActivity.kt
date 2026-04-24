package com.vibecode.nexus

import android.Manifest
import android.content.Intent
import android.os.Bundle
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Checklist
import androidx.compose.material.icons.filled.History
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
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.foundation.layout.padding
import androidx.compose.ui.Modifier
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
import com.vibecode.nexus.ui.screen.BrainDumpHistoryScreen
import com.vibecode.nexus.ui.screen.BrainDumpScreen
import com.vibecode.nexus.ui.screen.PairScreen
import com.vibecode.nexus.ui.screen.ProjectsScreen
import com.vibecode.nexus.ui.screen.SettingsScreen
import com.vibecode.nexus.ui.screen.TasksScreen
import com.vibecode.nexus.ui.screen.WelcomeScreen
import com.vibecode.nexus.ui.theme.NexusTheme
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow

data class BottomNavItem(
    val route: String,
    val label: String,
    val icon: ImageVector
)

class MainActivity : ComponentActivity() {

    private val bottomNavItems = listOf(
        BottomNavItem("braindump", "BrainDump", Icons.Default.Mic),
        BottomNavItem("history", "Verlauf", Icons.Default.History),
        BottomNavItem("tasks", "Tasks", Icons.Default.Checklist),
        BottomNavItem("projects", "Projekte", Icons.AutoMirrored.Filled.TrendingUp),
        BottomNavItem("settings", "Settings", Icons.Default.Settings),
    )

    // Holds a raw pairing URI that needs to be consumed by the UI layer.
    // Filled from incoming Intents (VIEW action with scheme "nexus"), drained
    // by a LaunchedEffect that calls ConnectionSettings.saveFromQr().
    private val pendingPairingUri = MutableStateFlow<String?>(null)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        consumePairingFromIntent(intent)
        setContent {
            NexusTheme {
                val navController = rememberNavController()
                val connectionSettings = remember { ConnectionSettings(this) }
                val apiClient = remember { NexusApiClient(connectionSettings) }
                val pendingUri by pendingPairingUri.collectAsState()
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

                // Consume deep-link pairings
                LaunchedEffect(pendingUri) {
                    val uri = pendingUri ?: return@LaunchedEffect
                    val ok = connectionSettings.saveFromQr(uri)
                    if (ok) {
                        isPaired = true
                        isConnected = apiClient.checkHealth()
                        val currentRoute = navController.currentDestination?.route
                        if (currentRoute in listOf("welcome", "pair")) {
                            navController.navigate("braindump") {
                                popUpTo("welcome") { inclusive = true }
                            }
                        }
                    } else {
                        Log.w("MainActivity", "Pairing deep-link could not be parsed: $uri")
                    }
                    pendingPairingUri.value = null
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
                    val startDest = if (connectionSettings.isPaired) "braindump" else "welcome"
                    NavHost(
                        navController = navController,
                        startDestination = startDest,
                        modifier = Modifier.padding(innerPadding)
                    ) {
                        composable("welcome") {
                            WelcomeScreen(
                                onContinue = { navController.navigate("pair") }
                            )
                        }
                        composable("pair") {
                            PairScreen(
                                connectionSettings = connectionSettings,
                                onPaired = {
                                    isPaired = true
                                    navController.navigate("braindump") {
                                        popUpTo("welcome") { inclusive = true }
                                    }
                                },
                                onBack = { navController.popBackStack() }
                            )
                        }
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
                        composable("history") {
                            BrainDumpHistoryScreen(apiClient = apiClient)
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

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        consumePairingFromIntent(intent)
    }

    private fun consumePairingFromIntent(intent: Intent?) {
        val data = intent?.data ?: return
        if (data.scheme == "nexus" && data.host == "pair") {
            pendingPairingUri.value = data.toString()
        }
    }
}

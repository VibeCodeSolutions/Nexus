package com.vibecode.nexus

import android.Manifest
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.core.content.ContextCompat
import androidx.core.content.PermissionChecker
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import com.vibecode.nexus.data.ConnectionSettings
import com.vibecode.nexus.data.NexusApiClient
import com.vibecode.nexus.speech.SpeechRecognizerManager
import com.vibecode.nexus.ui.screen.BrainDumpScreen
import com.vibecode.nexus.ui.screen.SettingsScreen
import com.vibecode.nexus.ui.theme.NexusTheme
import kotlinx.coroutines.delay

class MainActivity : ComponentActivity() {
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

                NavHost(navController = navController, startDestination = "braindump") {
                    composable("braindump") {
                        // Refresh paired state when returning from settings
                        LaunchedEffect(Unit) {
                            isPaired = connectionSettings.isPaired
                        }
                        BrainDumpScreen(
                            speechManager = speechManager,
                            apiClient = apiClient,
                            isPaired = isPaired,
                            isConnected = isConnected,
                            hasPermission = hasAudioPermission,
                            onRequestPermission = {
                                audioPermissionLauncher.launch(Manifest.permission.RECORD_AUDIO)
                            },
                            onNavigateToSettings = {
                                navController.navigate("settings")
                            }
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

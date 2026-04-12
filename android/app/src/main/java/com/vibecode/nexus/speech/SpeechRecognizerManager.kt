package com.vibecode.nexus.speech

import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.speech.RecognitionListener
import android.speech.RecognizerIntent
import android.speech.SpeechRecognizer
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

enum class RecognizerState {
    IDLE,
    LISTENING,
    ERROR
}

data class SpeechState(
    val recognizerState: RecognizerState = RecognizerState.IDLE,
    val partialText: String = "",
    val finalText: String = "",
    val errorMessage: String? = null,
    val isAvailable: Boolean = true
)

class SpeechRecognizerManager(private val context: Context) {

    private val _state = MutableStateFlow(SpeechState())
    val state: StateFlow<SpeechState> = _state.asStateFlow()

    private var recognizer: SpeechRecognizer? = null

    fun initialize() {
        val available = SpeechRecognizer.isRecognitionAvailable(context)
        _state.value = _state.value.copy(isAvailable = available)
        if (available) {
            recognizer = SpeechRecognizer.createSpeechRecognizer(context).apply {
                setRecognitionListener(listener)
            }
        }
    }

    fun startListening() {
        if (_state.value.recognizerState == RecognizerState.LISTENING) return
        if (!_state.value.isAvailable) {
            _state.value = _state.value.copy(
                errorMessage = "Spracherkennung nicht verfügbar. Offline?"
            )
            return
        }

        val intent = Intent(RecognizerIntent.ACTION_RECOGNIZE_SPEECH).apply {
            putExtra(RecognizerIntent.EXTRA_LANGUAGE_MODEL, RecognizerIntent.LANGUAGE_MODEL_FREE_FORM)
            putExtra(RecognizerIntent.EXTRA_PARTIAL_RESULTS, true)
            putExtra(RecognizerIntent.EXTRA_LANGUAGE, "de-DE")
        }

        _state.value = _state.value.copy(
            recognizerState = RecognizerState.LISTENING,
            partialText = "",
            errorMessage = null
        )
        recognizer?.startListening(intent)
    }

    fun stopListening() {
        recognizer?.stopListening()
    }

    fun destroy() {
        recognizer?.destroy()
        recognizer = null
    }

    private val listener = object : RecognitionListener {
        override fun onReadyForSpeech(params: Bundle?) {}
        override fun onBeginningOfSpeech() {}
        override fun onRmsChanged(rmsdB: Float) {}
        override fun onBufferReceived(buffer: ByteArray?) {}
        override fun onEndOfSpeech() {}

        override fun onPartialResults(partialResults: Bundle?) {
            val matches = partialResults
                ?.getStringArrayList(SpeechRecognizer.RESULTS_RECOGNITION)
            if (!matches.isNullOrEmpty()) {
                _state.value = _state.value.copy(partialText = matches[0])
            }
        }

        override fun onResults(results: Bundle?) {
            val matches = results
                ?.getStringArrayList(SpeechRecognizer.RESULTS_RECOGNITION)
            val text = if (!matches.isNullOrEmpty()) matches[0] else _state.value.partialText

            val current = _state.value.finalText
            val combined = if (current.isBlank()) text else "$current $text"

            _state.value = _state.value.copy(
                recognizerState = RecognizerState.IDLE,
                finalText = combined,
                partialText = ""
            )
        }

        override fun onError(error: Int) {
            val msg = when (error) {
                SpeechRecognizer.ERROR_NETWORK_TIMEOUT,
                SpeechRecognizer.ERROR_NETWORK -> "Netzwerkfehler — bist du online?"
                SpeechRecognizer.ERROR_NO_MATCH -> "Nichts erkannt. Nochmal versuchen."
                SpeechRecognizer.ERROR_SPEECH_TIMEOUT -> "Keine Sprache erkannt."
                else -> "Fehler bei Spracherkennung (Code $error)"
            }
            _state.value = _state.value.copy(
                recognizerState = RecognizerState.ERROR,
                errorMessage = msg
            )
        }

        override fun onEvent(eventType: Int, params: Bundle?) {}
    }
}

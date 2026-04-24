package com.vibecode.nexus.ui.screen

import android.content.Context
import com.google.mlkit.vision.barcode.common.Barcode
import com.google.mlkit.vision.codescanner.GmsBarcodeScannerOptions
import com.google.mlkit.vision.codescanner.GmsBarcodeScanning

fun startQrPairingScan(
    context: Context,
    onSuccess: (String) -> Unit,
    onCancel: () -> Unit = {},
    onFailure: (Exception) -> Unit = {}
) {
    val options = GmsBarcodeScannerOptions.Builder()
        .setBarcodeFormats(Barcode.FORMAT_QR_CODE)
        .build()
    val scanner = GmsBarcodeScanning.getClient(context, options)
    scanner.startScan()
        .addOnSuccessListener { barcode ->
            val raw = barcode.rawValue.orEmpty()
            if (raw.isNotEmpty()) onSuccess(raw) else onCancel()
        }
        .addOnCanceledListener { onCancel() }
        .addOnFailureListener { e -> onFailure(e) }
}

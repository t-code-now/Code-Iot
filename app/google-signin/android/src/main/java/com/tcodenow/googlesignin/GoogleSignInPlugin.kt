package com.tcodenow.googlesignin

import android.app.Activity
import android.graphics.Color
import android.view.ViewGroup
import android.webkit.WebView
import androidx.core.view.ViewCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.credentials.CredentialManager
import androidx.credentials.CustomCredential
import androidx.credentials.GetCredentialRequest
import androidx.credentials.exceptions.GetCredentialCancellationException
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import com.google.android.libraries.identity.googleid.GetSignInWithGoogleOption
import com.google.android.libraries.identity.googleid.GoogleIdTokenCredential
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

@InvokeArg
class SignInArgs {
    lateinit var clientId: String
}

/** Google's account sheet over the app (Android Credential Manager). */
@TauriPlugin
class GoogleSignInPlugin(private val activity: Activity) : Plugin(activity) {
    /**
     * Keeps the app out from under the phone's status bar and navigation bar.
     * Android draws apps edge to edge, so without this the clock, battery and
     * notification icons sit on top of the app's header (and catch its taps).
     * The web view is moved inside the safe area and the strips around it get
     * the app's navy, with light icons.
     */
    override fun load(webView: WebView) {
        super.load(webView)
        val window = activity.window
        window.decorView.setBackgroundColor(Color.parseColor("#121A38"))
        WindowCompat.getInsetsController(window, window.decorView).apply {
            isAppearanceLightStatusBars = false
            isAppearanceLightNavigationBars = false
        }
        ViewCompat.setOnApplyWindowInsetsListener(webView) { view, insets ->
            val bars = insets.getInsets(
                WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout()
            )
            val params = view.layoutParams
            if (params is ViewGroup.MarginLayoutParams) {
                if (params.topMargin != bars.top || params.bottomMargin != bars.bottom ||
                    params.leftMargin != bars.left || params.rightMargin != bars.right
                ) {
                    params.setMargins(bars.left, bars.top, bars.right, bars.bottom)
                    view.layoutParams = params
                }
            }
            insets
        }
        ViewCompat.requestApplyInsets(webView)
    }

    @Command
    fun signIn(invoke: Invoke) {
        val args = invoke.parseArgs(SignInArgs::class.java)
        val option = GetSignInWithGoogleOption.Builder(args.clientId).build()
        val request = GetCredentialRequest.Builder().addCredentialOption(option).build()
        val manager = CredentialManager.create(activity)
        CoroutineScope(Dispatchers.Main).launch {
            try {
                val credential = manager.getCredential(activity, request).credential
                if (credential is CustomCredential &&
                    credential.type == GoogleIdTokenCredential.TYPE_GOOGLE_ID_TOKEN_CREDENTIAL
                ) {
                    val google = GoogleIdTokenCredential.createFrom(credential.data)
                    val result = JSObject()
                    result.put("idToken", google.idToken)
                    invoke.resolve(result)
                } else {
                    invoke.reject("unexpected credential")
                }
            } catch (e: GetCredentialCancellationException) {
                invoke.reject("cancelled")
            } catch (e: Exception) {
                invoke.reject(e.message ?: e.javaClass.simpleName)
            }
        }
    }
}

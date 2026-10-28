package com.tcodenow.googlesignin

import android.app.Activity
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

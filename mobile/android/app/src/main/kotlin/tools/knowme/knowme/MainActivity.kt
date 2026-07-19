package tools.knowme.knowme

import android.util.Log
import io.flutter.embedding.android.FlutterActivity

class MainActivity : FlutterActivity() {
    companion object {
        private const val TAG = "KnowMeMainActivity"

        init {
            System.loadLibrary("gen_ui_ffi")
            Log.i(TAG, "Loaded gen_ui_ffi through System.loadLibrary")
        }
    }
}

package com.example.kongde

import android.Manifest
import android.content.Intent
import android.net.Uri
import android.content.ContentUris
import android.content.pm.PackageManager
import android.os.Build
import android.os.Bundle
import android.provider.MediaStore
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import com.ryanheise.audioservice.AudioServiceActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel

class MainActivity : AudioServiceActivity() {
    private val CHANNEL = "com.example.kongde/music"
    private val PLAYER_CHANNEL = "com.example.kongde/player"
    private val WIDGET_CHANNEL = "com.example.kongde/widget"
    private val READ_EXTERNAL_STORAGE_REQUEST_CODE = 1001
    private var pendingResult: MethodChannel.Result? = null
    private var flutterEngine: FlutterEngine? = null

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        this.flutterEngine = flutterEngine
        
        val methodChannel = MethodChannel(flutterEngine.dartExecutor.binaryMessenger, CHANNEL)
        methodChannel.setMethodCallHandler { call, result ->
            if (call.method == "getMusicList") {
                if (checkPermission()) {
                    val musicList = getMusicList()
                    result.success(musicList)
                } else {
                    pendingResult = result
                    requestPermission()
                }
            } else if (call.method == "getAlbumArt") {
                val albumId = call.argument<Long>("albumId") ?: 0
                val art = getAlbumArt(albumId)
                result.success(art)
            } else {
                result.notImplemented()
            }
        }

        val playerChannel = MethodChannel(flutterEngine.dartExecutor.binaryMessenger, PLAYER_CHANNEL)
        playerChannel.setMethodCallHandler { call, result ->
            result.notImplemented()
        }
        
        val widgetChannel = MethodChannel(flutterEngine.dartExecutor.binaryMessenger, WIDGET_CHANNEL)
        widgetChannel.setMethodCallHandler { call, result ->
            if (call.method == "updateWidget") {
                val word = call.argument<String>("word")
                if (word != null && word.isNotEmpty()) {
                    updateWidget(word)
                    result.success(null)
                } else {
                    result.error("INVALID_ARGUMENT", "Word is empty or null", null)
                }
            } else {
                result.notImplemented()
            }
        }
    }

    private fun checkPermission(): Boolean {
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            ContextCompat.checkSelfPermission(this, Manifest.permission.READ_MEDIA_AUDIO) == PackageManager.PERMISSION_GRANTED
        } else {
            ContextCompat.checkSelfPermission(this, Manifest.permission.READ_EXTERNAL_STORAGE) == PackageManager.PERMISSION_GRANTED
        }
    }

    private fun requestPermission() {
        val permissions = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            arrayOf(Manifest.permission.READ_MEDIA_AUDIO)
        } else {
            arrayOf(Manifest.permission.READ_EXTERNAL_STORAGE)
        }
        ActivityCompat.requestPermissions(this, permissions, READ_EXTERNAL_STORAGE_REQUEST_CODE)
    }

    override fun onRequestPermissionsResult(requestCode: Int, permissions: Array<out String>, grantResults: IntArray) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        if (requestCode == READ_EXTERNAL_STORAGE_REQUEST_CODE) {
            if (grantResults.isNotEmpty() && grantResults[0] == PackageManager.PERMISSION_GRANTED) {
                val musicList = getMusicList()
                pendingResult?.success(musicList)
            } else {
                pendingResult?.error("PERMISSION_DENIED", "Permission denied", null)
            }
            pendingResult = null
        }
    }

    private fun getMusicList(): List<Map<String, String?>> {
        val musicList = mutableListOf<Map<String, String?>>()
        val projection = arrayOf(
            MediaStore.Audio.Media._ID,
            MediaStore.Audio.Media.TITLE,
            MediaStore.Audio.Media.ARTIST,
            MediaStore.Audio.Media.ALBUM,
            MediaStore.Audio.Media.ALBUM_ID,
            MediaStore.Audio.Media.DURATION,
            MediaStore.Audio.Media.DATA
        )

        val selection = "${MediaStore.Audio.Media.IS_MUSIC} != 0"
        val sortOrder = "${MediaStore.Audio.Media.TITLE} ASC"

        android.util.Log.d("MainActivity", "Starting music query...")
        android.util.Log.d("MainActivity", "Selection: $selection")

        contentResolver.query(
            MediaStore.Audio.Media.EXTERNAL_CONTENT_URI,
            projection,
            selection,
            null,
            sortOrder
        )?.use { cursor ->
            android.util.Log.d("MainActivity", "Query returned cursor: ${cursor != null}")
            if (cursor == null) {
                android.util.Log.e("MainActivity", "Cursor is null!")
                return@use
            }
            
            android.util.Log.d("MainActivity", "Cursor count: ${cursor.count}")
            
            if (cursor.count == 0) {
                android.util.Log.w("MainActivity", "No music files found in MediaStore")
                return@use
            }
            
            val idColumn = cursor.getColumnIndexOrThrow(MediaStore.Audio.Media._ID)
            val titleColumn = cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.TITLE)
            val artistColumn = cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.ARTIST)
            val albumColumn = cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.ALBUM)
            val albumIdColumn = cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.ALBUM_ID)
            val durationColumn = cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.DURATION)
            val dataColumn = cursor.getColumnIndexOrThrow(MediaStore.Audio.Media.DATA)

            while (cursor.moveToNext()) {
                val id = cursor.getLong(idColumn)
                val title = cursor.getString(titleColumn)
                val artist = cursor.getString(artistColumn)
                val album = cursor.getString(albumColumn)
                val albumId = cursor.getLong(albumIdColumn)
                val duration = cursor.getLong(durationColumn)
                val data = cursor.getString(dataColumn)

                val durationText = formatDuration(duration)

                musicList.add(mapOf(
                    "id" to id.toString(),
                    "title" to title,
                    "artist" to artist,
                    "album" to album,
                    "albumId" to albumId.toString(),
                    "duration" to durationText,
                    "path" to data
                ))
            }
            
            android.util.Log.d("MainActivity", "Found ${musicList.size} songs")
        }

        return musicList
    }

    private fun formatDuration(durationMs: Long): String {
        val seconds = (durationMs / 1000) % 60
        val minutes = (durationMs / (1000 * 60)) % 60
        val hours = durationMs / (1000 * 60 * 60)

        return if (hours > 0) {
            String.format("%d:%02d:%02d", hours, minutes, seconds)
        } else {
            String.format("%d:%02d", minutes, seconds)
        }
    }

    private fun updateWidget(word: String) {
        val prefs = getSharedPreferences("CollinsWidgetPrefs", MODE_PRIVATE)
        prefs.edit().putString("word_0", word).apply()
        CollinsDictionaryWidget.updateAllWidgets(this)
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        handleIntent(intent)
    }

    private fun handleIntent(intent: Intent) {
        val data = intent.data
        val route = intent.getStringExtra("route")
        var word: String? = null

        if (data != null && data.scheme == "kongde") {
            if (data.host == "collins_dict") {
                word = data.getQueryParameter("word")
            }
        } else {
            word = intent.getStringExtra("word")
        }

        if ((route == "/collins_dict" || data?.host == "collins_dict") && word != null) {
            flutterEngine?.let { engine ->
                val channel = MethodChannel(engine.dartExecutor.binaryMessenger, "com.example.kongde/navigation")
                channel.invokeMethod("navigate", mapOf(
                    "route" to "/collins_dict",
                    "word" to word
                ))
            }
        }
    }

    private fun getAlbumArt(albumId: Long): ByteArray? {
        return try {
            val uri = ContentUris.withAppendedId(MediaStore.Audio.Albums.EXTERNAL_CONTENT_URI, albumId)
            val cursor = contentResolver.query(uri, arrayOf(MediaStore.Audio.Albums.ALBUM_ART), null, null, null)
            cursor?.use {
                if (it.moveToFirst()) {
                    val path = it.getString(0)
                    if (path != null) {
                        val file = java.io.File(path)
                        if (file.exists()) file.readBytes() else null
                    } else null
                } else null
            }
        } catch (e: Exception) {
            null
        }
    }
}

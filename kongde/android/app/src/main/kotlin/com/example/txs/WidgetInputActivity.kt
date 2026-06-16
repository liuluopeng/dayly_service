package com.example.kongde

import android.app.Activity
import android.appwidget.AppWidgetManager
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.widget.Button
import android.widget.EditText
import android.widget.Toast
import androidx.appcompat.app.AppCompatActivity

class WidgetInputActivity : AppCompatActivity() {

    private var appWidgetId = AppWidgetManager.INVALID_APPWIDGET_ID

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val extras = intent.extras
        if (extras != null) {
            appWidgetId = extras.getInt(
                AppWidgetManager.EXTRA_APPWIDGET_ID,
                AppWidgetManager.INVALID_APPWIDGET_ID
            )
        }

        if (appWidgetId == AppWidgetManager.INVALID_APPWIDGET_ID) {
            finish()
            return
        }

        setContentView(R.layout.widget_input_activity)

        val editText = findViewById<EditText>(R.id.word_input)
        val searchButton = findViewById<Button>(R.id.search_button)
        val cancelButton = findViewById<Button>(R.id.cancel_button)

        searchButton.setOnClickListener {
            val word = editText.text.toString().trim()
            if (word.isNotEmpty()) {
                saveWordAndSearch(word)
            } else {
                Toast.makeText(this, "请输入单词", Toast.LENGTH_SHORT).show()
            }
        }

        cancelButton.setOnClickListener {
            finish()
        }
    }

    private fun saveWordAndSearch(word: String) {
        val prefs = getSharedPreferences("CollinsWidgetPrefs", Context.MODE_PRIVATE)
        prefs.edit().putString("word_$appWidgetId", word).apply()

        CollinsDictionaryWidget.updateAllWidgets(this)

        openCollinsDictionary(word)

        finish()
    }

    private fun openCollinsDictionary(word: String) {
        try {
            val intent = Intent(Intent.ACTION_VIEW).apply {
                data = Uri.parse("kongde://collins_dict?word=$word")
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TOP)
            }
            startActivity(intent)
        } catch (e: Exception) {
            Toast.makeText(this, "无法打开柯林斯词典", Toast.LENGTH_SHORT).show()
        }
    }
}

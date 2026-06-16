package com.example.kongde

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Context.MODE_PRIVATE
import android.content.Intent
import android.widget.Toast

class WidgetUpdateReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        val word = intent.getStringExtra("word") ?: ""
        
        if (word.isNotEmpty()) {
            val prefs = context.getSharedPreferences("CollinsWidgetPrefs", MODE_PRIVATE)
            prefs.edit().putString("word_0", word).apply()
            
            CollinsDictionaryWidget.updateAllWidgets(context)
            
            Toast.makeText(context, "小组件已更新: $word", Toast.LENGTH_SHORT).show()
        }
    }
}

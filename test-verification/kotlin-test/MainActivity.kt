package com.example

class MainActivity {
    private val counter = 0
    
    fun onCreate() {
        println("App started")
    }
    
    fun handleClick() {
        println("Clicked\!")
    }
}

data class User(val id: Int, val name: String)

enum class State {
    IDLE, LOADING, ERROR
}
EOF < /dev/null
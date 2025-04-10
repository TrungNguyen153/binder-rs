package com.example.binderserver

class BinderServer {
    companion object {
        init {
            System.loadLibrary("binder_rs")
        }

        external fun loadService()
    }
}
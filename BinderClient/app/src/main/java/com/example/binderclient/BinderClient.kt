package com.example.binderclient

class BinderClient {
    companion object {
        init {
            System.loadLibrary("binder_rs")
        }
        external fun loadService()
    }
}
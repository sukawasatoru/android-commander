/*
 * Copyright 2021 sukawasatoru
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package jp.tinyport.androidcommander.server

import android.os.SystemClock
import android.text.TextUtils
import android.view.KeyEvent
import java.util.Scanner

fun main() {
    println("Hello")

    val serviceManager = ServiceManagerWrapper()
    val inputManager = serviceManager.inputManager
    val scanner = Scanner(System.`in`)

    var lastCommand: ClientCommand? = null
    var repeatCount = 0

    while (true) {
        val command = try {
            parseLine(scanner.nextLine()) ?: continue
        } catch (e: NoSuchElementException) {
            break
        }

        if (command.action == KeyAction.Down && command == lastCommand) {
            repeatCount += 1
        } else {
            repeatCount = 0
        }

        val eventTime = SystemClock.uptimeMillis()
        inputManager.injectInputEvent(
            KeyEvent(
                eventTime,
                eventTime,
                command.action.code,
                command.code.code,
                repeatCount
            ),
            InputManagerWrapper.INJECT_INPUT_EVENT_MODE_ASYNC,
        )

        lastCommand = command
    }

    println("Bye")
}

private fun parseLine(line: String): ClientCommand? {
    val lineSegments = TextUtils.split(line, " ")
    if (lineSegments.size != 2) {
        println("unexpected format: $line")
        return null
    }

    val (actionString, codeString) = lineSegments

    return ClientCommand(
        action = KeyAction.from(actionString) ?: run {
            println("unexpected action: $actionString")
            return null
        },
        code = KeyCode.from(codeString) ?: run {
            println("unexpected code: $codeString")
            return null
        },
    )
}

enum class KeyAction {
    Down,
    Up;

    companion object {
        fun from(value: String): KeyAction? {
            return values().find { it.id == value }
        }
    }

    val id: String
        get() = when (this) {
            Down -> "down"
            Up -> "up"
        }

    val code: Int
        get() = when (this) {
            Down -> KeyEvent.ACTION_DOWN
            Up -> KeyEvent.ACTION_UP
        }
}

data class ClientCommand(
    val action: KeyAction,
    val code: KeyCode,
)

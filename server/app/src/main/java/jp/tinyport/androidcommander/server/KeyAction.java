/*
 * Copyright 2021, 2025 sukawasatoru
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

package jp.tinyport.androidcommander.server;

import android.view.KeyEvent;

public enum KeyAction {
    Down("down", KeyEvent.ACTION_DOWN),
    Up("up", KeyEvent.ACTION_UP);

    public static KeyAction from(String value) {
        for (KeyAction action : values()) {
            if (action.id.equals(value)) {
                return action;
            }
        }
        return null;
    }

    public final String id;
    public final int code;

    KeyAction(String id, int code) {
        this.id = id;
        this.code = code;
    }
}

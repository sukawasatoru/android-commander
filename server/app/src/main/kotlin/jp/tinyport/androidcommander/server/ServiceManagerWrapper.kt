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

import android.content.Context
import android.os.IBinder
import android.os.IInterface

class ServiceManagerWrapper {
    private val getService = Class.forName("android.os.ServiceManager")
            .getDeclaredMethod("getService", String::class.java)

    val inputManager = InputManagerWrapper(
            Class.forName("android.hardware.input.IInputManager\$Stub")
                    .getMethod("asInterface", IBinder::class.java)
                    .invoke(null, getService.invoke(null, Context.INPUT_SERVICE)) as IInterface
    )
}

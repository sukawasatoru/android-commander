Android Commander
=================

https://user-images.githubusercontent.com/12950393/147870786-2aa12ae9-66ab-4a49-9337-e32f62b1afc3.mov

Overview
--------

![Sequence](https://user-images.githubusercontent.com/12950393/147868824-48336422-ef39-4292-a915-bbc6fc5f9ea5.png)

<details>
<summary>sd</summary>

```
actor:Actor
client:Client "Android Commander (Client)"
/server:Server "Android Commander (Server)"
os:OS[a]

client:os.adb push android-commander-server /data/local/tmp
client:os.adb shell app_process android-commander-server
os:server.new
actor:client.
client:server."down KEYCODE_DPAD_LEFT" (via. stdin)
  server:KeyEvent(ACTION_DOWN,KEYCODE_DPAD_LEFT)=server.parse()
  server[1]:_
  server:os.injectInputEvent(KeyEvent, INJECT_INPUT_EVENT_MODE_ASYNC)

client[1]:stop
```
</details>

LICENSE
-------

```
   Copyright 2020, 2021, 2022 sukawasatoru

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
```

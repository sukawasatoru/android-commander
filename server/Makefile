SHELL = /bin/bash
.SUFFIXES:

GRADLEW = ./gradlew
REN = mv

ifeq ($(OS),Windows_NT)
    SHELL = cmd
    GRADLEW = gradlew.bat
    REN = REN
endif

VERSION_BUILD_TOOLS=36.0.0

.PHONY: all
all: compile

.PHONY: compile
compile:
	$(GRADLEW) :app:assembleRelease

# Windows requires the following patch:
# --- a/build-tools/31.0.0/d8.bat
# +++ b/build-tools/31.0.0/d8.bat
# @@ -106,4 +106,4 @@
#  :endArgs
#
#  set javaOpts=%javaOpts% %defaultXmx% %defaultXss%
# -call "%java_exe%" %javaOpts% -Djava.ext.dirs="%frameworkdir%" -cp "%jarpath%" com.android.tools.r8.D8 %params%
# +call "%java_exe%" %javaOpts% -classpath "%frameworkdir%" -cp "%jarpath%" com.android.tools.r8.D8 %params%
	$(ANDROID_HOME)/build-tools/$(VERSION_BUILD_TOOLS)/d8 --output app/build/outputs/android-commander-server.zip app/build/intermediates/aar_main_jar/release/syncReleaseLibJars/classes.jar
	cd app/build/outputs && $(REN) android-commander-server.zip android-commander-server

.PHONY: clean
clean:
	./gradlew clean

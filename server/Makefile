SHELL = /bin/bash
.SUFFIXES:

MV = mv

ifeq ($(OS),Windows_NT)
    SHELL = cmd
    MV = REN
endif

VERSION_BUILD_TOOLS=31.0.0

.PHONY: all
all: compile

.PHONY: compile
compile:
	./gradlew :app:assembleRelease
	$(ANDROID_HOME)/build-tools/$(VERSION_BUILD_TOOLS)/d8 --output app/build/outputs/android-commander-server.zip app/build/intermediates/aar_main_jar/release/classes.jar
	$(MV) app/build/outputs/android-commander-server.zip app/build/outputs/android-commander-server

.PHONY: clean
clean:
	./gradlew clean

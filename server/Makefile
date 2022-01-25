SHELL = /bin/bash
.SUFFIXES:

GRADLEW = ./gradlew
REN = mv

ifeq ($(OS),Windows_NT)
    SHELL = cmd
    GRADLEW = gradlew.bat
    REN = REN
endif

VERSION_BUILD_TOOLS=31.0.0

.PHONY: all
all: compile

.PHONY: compile
compile:
	$(GRADLEW) :app:assembleRelease
	$(ANDROID_HOME)/build-tools/$(VERSION_BUILD_TOOLS)/d8 --output app/build/outputs/android-commander-server.zip app/build/intermediates/aar_main_jar/release/classes.jar
	cd app/build/outputs && $(REN) android-commander-server.zip android-commander-server

.PHONY: clean
clean:
	./gradlew clean

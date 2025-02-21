SECRET_EXISTS := $(shell [ -e Makefile.secret ] && echo "EXISTS")

ifneq ($(SECRET_EXISTS), EXISTS)
IGNORE := $(shell echo -en "KEYSTORE_PATH := path/to/your/keystore.jks\nKEYSTORE_PASS_PATH := path/to/your/password.pwd\nKEY_ALIAS := alias of your key\nKEY_PASS_PATH := /path/to/your/key_password.pwd" > Makefile.secret)
$(error Makefile.secret created, please fill in before continuing)
endif

include Makefile.secret

ifndef KEYSTORE_PATH
$(error KEYSTORE_PATH not set, please create a Makefile.secret file and set the variable there)
endif

ifndef KEYSTORE_PASS_PATH
$(error KEYSTORE_PASS_PATH not set, please create a Makefile.secret file and set the variable there)
endif

ifndef KEY_ALIAS
$(error KEY_ALIAS not set, please create a Makefile.secret file and set the variable there)
endif

ifndef KEY_PASS_PATH
$(error 'KEY_PASS_PATH not set, please create a Makefile.secret file and set the variable there')
endif

APKS_DIR := target/apks

.PHONY apks: $(APKS_DIR)/flashbang.apks

$(APKS_DIR)/flashbang.apks: | $(APKS_DIR)
	dx bundle --platform android -r
	java -jar bundletool.jar build-apks \
		--bundle target/dx/flashbang/release/android/app/app/build/outputs/bundle/release/Flashbang-aarch64.aab \
		--output $(APKS_DIR)/flashbang.apks \
		--overwrite \
		--ks $(KEYSTORE_PATH) \
		--ks-pass file:$(KEYSTORE_PASS_PATH) \
		--ks-key-alias $(KEY_ALIAS) \
		--key-pass file:$(KEY_PASS_PATH)

.PHONY install: apks
	java -jar bundletool.jar install-apks --apks $(APKS_DIR)/flashbang.apks

$(APKS_DIR):
	mkdir -p $(APKS_DIR)

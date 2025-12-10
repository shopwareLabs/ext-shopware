UNAME_S := $(shell uname -s)

build:
ifeq ($(UNAME_S),Darwin)
	export PKG_CONFIG_PATH="/opt/homebrew/lib/pkgconfig:$(PKG_CONFIG_PATH)" && \
	export LIBRARY_PATH="/opt/homebrew/lib:$(LIBRARY_PATH)" && \
	cargo build --release && \
	cp target/release/libext_shopware.dylib target/release/libext_shopware.so
else
	cargo build --release
endif

build-debug:
ifeq ($(UNAME_S),Darwin)
	export PKG_CONFIG_PATH="/opt/homebrew/lib/pkgconfig:$(PKG_CONFIG_PATH)" && \
	export LIBRARY_PATH="/opt/homebrew/lib:$(LIBRARY_PATH)" && \
	cargo build && \
	cp target/debug/libext_shopware.dylib target/debug/libext_shopware.so
else
	cargo build
endif

test: build
	php -dextension=target/release/libext_shopware.so vendor/bin/phpunit

test-debug: build-debug
	php -dextension=target/debug/libext_shopware.so vendor/bin/phpunit

bench: build
	./vendor/bin/phpbench run benchmarks/ --report=default
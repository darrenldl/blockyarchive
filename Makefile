SRCFILES = build.rs \
	src/*.rs \
	src/bin/*.rs \
	src/rs_codec/*.rs \
	src/sbx_block/*.rs

.PHONY: all
all : bin

.PHONY: bin
bin :
	cargo build

.PHONY: format
format :
	rustfmt $(SRCFILES)

.PHONY: test
test :
	cd tests && ./dev_tests.sh

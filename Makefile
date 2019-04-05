SRCFILES = src/*.ml* \
	src/bin/*.ml \
	src/rs_codec/*.ml* \
	src/sbx_block/*.ml*

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

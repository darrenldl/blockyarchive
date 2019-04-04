SRCFILES = src/*.rs \
	src/bin/*.rs \
	src/rs_codec/*.rs \
	src/sbx_block/*.rs

.PHONY: format
format :
	rustfmt $(SRCFILES)

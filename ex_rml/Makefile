SHELL := /usr/bin/fish

all: build

# Note: this rule is written for a fish-like terminal (using parenthesis syntax for running a subcommand)
# Consider changing the shell (at the top of this file), or editing the `ocamlc` call
build: example.rml
	rmlc example.rml
	ocamlc -o example -I (rmlc -where) unix.cma rmllib.cma example.ml

clean:
	rm -f *.rzi *.ml *.cm?
	rm -f example


# litgraph-emacs -- developer targets.
#
#   make test      ERT unit tests (process layer stubbed) plus integration
#                  tests against the real `litgraph' binary when installed
#   make compile   byte-compile with warnings as errors
#   make lint      checkdoc and package-lint
#   make all       compile, lint, test
#
# package-lint is looked up in PACKAGE_LINT_DIR, else in .deps/, else
# installed there from MELPA (needs network).

EMACS ?= emacs
BATCH  = $(EMACS) -Q --batch -L .
SRC    = litgraph.el litgraph-view.el litgraph-graph.el litgraph-org.el \
         litgraph-scenario.el litgraph-menu.el litgraph-timeline.el \
         ob-litgraph.el
TESTS  = $(wildcard test/*-test.el)
PACKAGE_LINT_DIR ?=

.PHONY: all test compile lint checkdoc package-lint autoloads clean deps

all: compile lint test

compile: clean-elc
	$(BATCH) --eval '(setq byte-compile-error-on-warn t)' \
	  -f batch-byte-compile $(SRC)

clean-elc:
	rm -f *.elc

test: clean-elc
	$(BATCH) -L test $(foreach t,$(TESTS),-l $(t)) \
	  -f ert-run-tests-batch-and-exit

lint: checkdoc package-lint

checkdoc:
	$(BATCH) -l scripts/lint.el -f litgraph-lint-checkdoc $(SRC)

package-lint: deps
	$(BATCH) -l scripts/lint.el --eval '(litgraph-lint-package-lint "$(PACKAGE_LINT_DIR)")' $(SRC)

deps:
	@true

autoloads:
	$(BATCH) --eval '(loaddefs-generate default-directory "litgraph-autoloads.el")' || \
	$(BATCH) --eval '(make-directory-autoloads default-directory "litgraph-autoloads.el")'

clean:
	rm -f *.elc litgraph-autoloads.el
	rm -rf .deps
